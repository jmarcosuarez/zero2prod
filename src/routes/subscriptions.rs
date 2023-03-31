use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::{query, PgPool};
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
        request_id=%Uuid::new_v4(),
        subscriber_email =%form.email,
        subscriber_name =%form.name
    )
)]
pub async fn subscribe(
    form: web::Form<FormData>,
    // Retrieving a connection from the application state
    pool: web::Data<PgPool>,
) -> HttpResponse {
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
