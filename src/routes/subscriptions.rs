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

// Creates a span at the beginning of the function invocation and
// automatically attaches all instruments passed to the function to
// the context of the span
#[tracing::instrument(
    name="Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    // Retrieving a connection from the application state
    pool: web::Data<PgPool>,
) -> HttpResponse {
    if !is_valid_name(&form.name) {
        return HttpResponse::BadRequest().finish();
    }

    // `Result` has 2 variant: `Ok` and `Err`.
    // The first for successes, the second for failures
    // We use a `match` statement to choose what to do based on the outcome
    match insert_subscriber(&pool, &form).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    // Retrieving a connection from the application state
    form: &FormData,
) -> Result<(), sqlx::Error> {
    query!(
        r#"
                    INSERT INTO subscriptions (id, email, name, subscribed_at) 
                    VALUES ($1, $2, $3, $4)
                "#,
        Uuid::new_v4(),
        form.email,
        form.name,
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
