use crate::domain::SubscriberEmail;
use crate::email_client::EmailClient;
use crate::routes::error_chain_fmt;
use actix_web::http::StatusCode;
use actix_web::ResponseError;
use actix_web::{web, HttpResponse};
use anyhow::Context;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct BodyData {
    title: String,
    content: Content,
}

#[derive(serde::Deserialize)]
pub struct Content {
    html: String,
    text: String,
}

#[derive(thiserror::Error)]
pub enum PublishError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

// Same logic to get the full error chain on `Debug`
impl std::fmt::Debug for PublishError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for PublishError {
    fn status_code(&self) -> StatusCode {
        match self {
            PublishError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

// Naive approach:
// Retrieve the newsletter issue from details form the body of the incoming API call.
// Fetch the list of confirmed subscribers from the database.
// Iterate through the whole list:
//  - Get the subscriber email.
//  - Send an email out via Postmark.

// The actix extractor to parse bodyData out od call is JSON.
// when dealing with an HTML form it would be `application/x-ww-form-urlencoded`
pub async fn publish_newsletter(
    body: web::Json<BodyData>,
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
) -> Result<HttpResponse, PublishError> {
    let subscribers = get_confirmed_subscribers(&pool).await?;
    for subscriber in subscribers {
        // The subscriber forces us to handle both the happy and the unhappy case
        match subscriber {
            Ok(subscriber) => {
                email_client
                    .send_email(
                        &subscriber.email,
                        &body.title,
                        &body.content.html,
                        &body.content.text,
                    )
                    .await
                    // close relative of `context` covert `error` variant of `Result` to `anyhow::Error`
                    // if the context you are adding has a runtime cost - use `with_context` (it is lazy)
                    // Using `context` would allocate that string everytime we send an email
                    // with `with_context` instead, we only would be invoke format! is the delivery fails!
                    .with_context(|| {
                        format!("Failed to send newsletter issue to {}", subscriber.email)
                    })?;
            }
            Err(error) => {
                tracing::warn!(
                    // We record the error chain as a structured field
                    // on the log record.
                    error.cause.chain = ?error,
                    // Using `\` to split a long string literal over
                    // two lines, without creating a `\n` character.
                    "Skipping a confirmed subscriber. \
                    Their stored contact details are invalid",
                );
            }
        }
    }
    Ok(HttpResponse::Ok().finish())
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
