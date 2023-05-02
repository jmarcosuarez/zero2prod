use crate::authentication::UserId;
use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::idempotency::IdempotencyKey;
use crate::idempotency::{get_saved_response, save_response};
use crate::utils::e400;
use crate::utils::{e500, see_other};
use actix_web::web::ReqData;
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use anyhow::Context;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct FormData {
    title: String,
    text_content: String,
    html_content: String,
    idempotency_key: String,
}

// Naive approach:
// Retrieve the newsletter issue from details form the body of the incoming API call.
// Fetch the list of confirmed subscribers from the database.
// Iterate through the whole list:
//  - Get the subscriber email.
//  - Send an email out via Postmark.

// The actix extractor to parse bodyData out od call is JSON.
// when dealing with an HTML form it would be `application/x-ww-form-urlencoded`
#[tracing::instrument(
    name="Publish a newsletter issue",
    skip(form, pool, email_client, user_id),
    fields(user_id=%*user_id)
)]
pub async fn publish_newsletter(
    form: web::Json<FormData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    // Inject the user id extracted from the user session
    user_id: ReqData<UserId>,
) -> Result<HttpResponse, actix_web::Error> {
    let user_id = user_id.into_inner();
    // We must destructure the form to avoid upsetting the borrow-checker
    let FormData {
        title,
        text_content,
        html_content,
        idempotency_key,
    } = form.0;
    let idempotency_key: IdempotencyKey = idempotency_key.try_into().map_err(e400)?;
    // Return early if we have a saved response in the database
    if let Some(saved_response) = get_saved_response(&pool, &idempotency_key, *user_id)
        .await
        .map_err(e500)?
    {
        FlashMessage::info("The newsletter issue have been published!").send();
        return Ok(saved_response);
    }
    let subscribers = get_confirmed_subscribers(&pool).await.map_err(e500)?;
    for subscriber in subscribers {
        // The subscriber forces us to handle both the happy and the unhappy case
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(&subscriber.email, &title, &html_content, &text_content)
                    .await
                    // close relative of `context` covert `error` variant of `Result` to `anyhow::Error`
                    // if the context you are adding has a runtime cost - use `with_context` (it is lazy)
                    // Using `context` would allocate that string everytime we send an email
                    // with `with_context` instead, we only would be invoke format! is the delivery fails!
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })
                    .map_err(e500)?;
            }
            Err(error) => {
                tracing::warn!(
                    // We record the error chain as a structured field
                    // on the log record.
                    error.cause.chain = ?error,
                    error.message = %error,
                    // Using `\` to split a long string literal over
                    // two lines, without creating a `\n` character.
                    "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid",
                );
            }
        }
    }
    FlashMessage::info("The newsletter issue have been published!").send();
    let response = see_other("/admin/newsletters");
    let response = save_response(&pool, &idempotency_key, *user_id, response)
        .await
        .map_err(e500)?;
    Ok(response)
}

struct ConfirmedSubscriber {
    email: SubscriberEmail,
}

#[tracing::instrument(name = "Get confirmed subscribers", skip(pool))]
async fn get_confirmed_subscribers(
    pool: &PgPool,
    // We are returning a `Vec` of `Results`'s in the happy case.
    // This allows the caller to bubble up the errors due to network issues or other
    // transient failures using the `?` operator, while the compiler
    // forces them to handle the subtler mapping error.
    // See http:://sled.rs/errors.html for a deep-dive about this technique.
) -> Result<Vec<Result<ConfirmedSubscriber, anyhow::Error>>, anyhow::Error> {
    // We only need `Row` to map the coming out of this query.
    // Nesting its definition inside the function itself is a simple way
    // to clearly communicate this coupling (and to ensure it doesn't
    // get used elsewhere by mistake).
    // struct Row { // removed later version
    //     email: String,
    // }

    // `query_as` maps the retrieved rows to the type specified as its first argument
    // We are minimizing the amount of data  we are fetching form DB (email only).
    // Less work for the DB and less data over the network!
    // Changed to query!
    let rows = sqlx::query!(
        // Row,
        r#"
        SELECT email
        FROM subscriptions
        WHERE status = 'confirmed'
        "#,
    )
    .fetch_all(pool)
    .await?;

    // Map into the domain type
    let confirmed_subscribers = rows
        .into_iter()
        .map(|r| match SubscriberEmail::parse(r.email) {
            Ok(email) => Ok(ConfirmedSubscriber { email }),
            Err(error) => Err(anyhow::anyhow!(error)),
        })
        // Used above instead!
        // You might argue that all the emails stored in database are necessarily valid.
        // We choose to validate again since these emails were market as valid by
        // a previous version of our app, which now could have changed.
        // We can either skip validation or process all - we need half-way - just emit a warning.
        // .filter_map(|r| match SubscriberEmail::parse(r.email) {
        //     Ok(email) => Some(ConfirmedSubscriber { email }),
        //     Err(error) => {
        //         tracing::warn!(
        //             "A confirmed subscriber is using an invalid email address.\n{}.",
        //             error
        //         );
        //         None
        //     }
        // })
        .collect();

    Ok(confirmed_subscribers)
}
