use actix_web::{web, HttpResponse};
use chrono::Utc;
use sqlx::PgPool;
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
    // `Result` has 2 variant: `Ok` and `Err`.
    // The first for successes, the second for failures
    // We use a `match` statement to choose what to do based on the outcome
    match sqlx::query(
        r#"
                    INSERT INTO subscriptions (id, email, name, subscribed_at) 
                    VALUES ($1, $2, $3, $4)
                "#,
    )
    .bind(Uuid::new_v4())
    .bind(&form.email)
    .bind(&form.name)
    .bind(Utc::now())
    // Using the pool as a replacement for PgConnection
    .execute(pool.as_ref())
    .await
    {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to excecute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}
