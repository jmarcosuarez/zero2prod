use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};

use crate::email_client::EmailClient;
use crate::startup::ApplicationBaseUrl;
use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::{query, PgPool};
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

// NOTE: `TryFrom/TryInto` implementation instead of this!!!
// pub fn parse_subscriber(form: FormData) -> Result<NewSubscriber, String> {
//     let name = SubscriberName::parse(form.name)?;
//     let email = SubscriberEmail::parse(form.email)?;
//     Ok(NewSubscriber { email, name })
// }

// The Rust standard lib provides a few traits to deal with conversions
// By implementing `TryFrom/TryInto` we are just making our intent clear.
// We are spelling "This is a type conversion"
impl TryFrom<FormData> for NewSubscriber {
    type Error = String;

    fn try_from(value: FormData) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;
        Ok(Self { email, name })
    }
}
// Creates a span at the beginning of the function invocation and
// automatically attaches all instruments passed to the function to
// the context of the span
#[tracing::instrument(
    name="Adding a new subscriber",
    skip(form, pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    // Retrieving a connection from the application state
    pool: web::Data<PgPool>,
    // Get the email_client form the app context
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
) -> HttpResponse {
    // We implemented `TryFrom` but we are calling `.try_into()`
    // `TryFrom implementation  gives you this for free
    // ` form.0.try_into()` equals `NewSubscriber::try_from(from.0)`
    // is just a mather of taste really!!
    let new_subscriber = match form.0.try_into() {
        Ok(form) => form,
        // Return early if the email is invalid, with a 400
        Err(_) => return HttpResponse::BadRequest().finish(),
    };

    if insert_subscriber(&pool, &new_subscriber).await.is_err() {
        return HttpResponse::InternalServerError().finish();
    }

    if send_confirmation_email(&email_client, new_subscriber, &base_url.0)
        .await
        .is_err()
    {
        return HttpResponse::InternalServerError().finish();
    }
    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Send a confirmation email to new subscriber",
    skip(email_client, new_subscriber, base_url)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
    base_url: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}/subscriptions/confirm?subscription_token=mytoken",
        base_url
    );
    // Subscriptions will send an email
    let html_body = &format!(
        "Welcome to our newsletter!<br />\
    Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    let plain_body = &format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );

    email_client
        .send_email(new_subscriber.email, "Welcome!", html_body, plain_body)
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]

pub async fn insert_subscriber(
    pool: &PgPool,
    // Retrieving a connection from the application state
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    query!(
        r#"
                    INSERT INTO subscriptions (id, email, name, subscribed_at, status) 
                    VALUES ($1, $2, $3, $4, 'pending_confirmation')
                "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    // Using the pool as a replacement for PgConnection
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
        // Using the `?` operator to return early
        // if the function failed, returning a sqlx::Error
    })?;

    Ok(())
}

/// Returns `true` if the input satisfies all our validation constrains
/// on subscribers names, `false` otherwise
pub fn is_valid_name(s: &str) -> bool {
    // `.trim() returns a view over the input `s` without trailing
    // whitespace-like characters.
    // `.is_empty` checks if the view contains any character
    let is_empty_or_whitespace = s.trim().is_empty();

    // A ghapheme is defined by the unicode standard as a "user-perceived"
    // character. `ñ` is a single grapheme, but it is composed of 2 characters (`n` and `~`)

    // `graphemes` returns an iterator over the graphemes in the input `s`
    // `true` specifies that we want to use the extended grapheme definition set,
    // the recommended one.
    let is_too_long = s.graphemes(true).count() > 256;

    // Iterate over all characters iun the input `s` to check if any of them matches
    // one of the characters in the forbidden array.
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contain_forbidden_characters = s.chars().any(|g| forbidden_characters.contains(&g));

    // Return `false` if any of our conditions have been violated
    !(is_empty_or_whitespace || is_too_long || contain_forbidden_characters)
}
