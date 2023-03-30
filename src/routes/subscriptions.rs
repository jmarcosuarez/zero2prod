use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::{query, PgPool};
use tracing::Instrument;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct FormData {
    email: String,
    name: String,
}

pub async fn subscribe(
    form: web::Form<FormData>,
    // Retrieving a connection from the application state
    pool: web::Data<PgPool>,
) -> HttpResponse {
    // Generates a random unique identifier to properly correlate logs
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
            "Adding a new subscriber.",
    %request_id,
    subscriber_email = %form.email,
    subscriber_name = %form.name,
        );
    let _request_span_guard = request_span.enter();

    // we do not call `.enter` on query_span!
    // `.instrument` takes care of it at the right moments
    // in the query future lifetime
    let query_span = tracing::info_span!("Saving new subscriber details in the database");

    // `Result` has 2 variant: `Ok` and `Err`.
    // The first for successes, the second for failures
    // We use a `match` statement to choose what to do based on the outcome
    match query!(
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
    .execute(pool.as_ref())
    // First we attach the instrumentation, then we `.await` it
    .instrument(query_span)
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            // yes, this error log falls outside of the `query_span`
            // we will fix it later
            tracing::error!(
                "Request id {} - Failed to execute query: {:?}",
                request_id,
                e
            );
            HttpResponse::InternalServerError().finish()
        }
    }
}
