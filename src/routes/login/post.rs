use crate::authentication::AuthError;
use crate::authentication::{validate_credentials, Credentials};
use crate::routes::error_chain_fmt;
use crate::session_state::TypedSession;

use actix_web::error::InternalError;
use actix_web::http::header::LOCATION;
use actix_web::{web, HttpResponse};
use actix_web_flash_messages::FlashMessage;
use secrecy::Secret;
use sqlx::PgPool;

#[derive(serde::Deserialize)]
pub struct FormData {
    username: String,
    password: Secret<String>,
}

#[tracing::instrument(
    skip(form, pool, session),
    fields(username=tracing::field::Empty, user_id=tracing::field::Empty)
)]
pub async fn login(
    form: web::Form<FormData>,
    pool: web::Data<PgPool>,
    session: TypedSession,
    // `actix_web::error::InternalError` can be returned as an error from a request handler
    // Otherwise we would have missed propagating upstream the error context
) -> Result<HttpResponse, InternalError<LoginError>> {
    let credentials = Credentials {
        username: form.0.username,
        password: form.0.password,
    };

    match validate_credentials(credentials, &pool).await {
        Ok(user_id) => {
            tracing::Span::current().record("user_id", &tracing::field::display(&user_id));
            // Rotate the session token when the user logs in - Prevents session fixation attacks
            session.renew();
            // We store the user identifier into the session
            // - then will retrieve from the session in state in admin_dashboard
            session
                .insert_user_id(user_id)
                .map_err(|e| login_redirect(LoginError::UnexpectedError(e.into())))?;
            Ok(HttpResponse::SeeOther()
                .insert_header((LOCATION, "/admin/dashboard"))
                .finish())
        }
        Err(e) => {
            let e = match e {
                AuthError::InvalidCredentials(_) => LoginError::AuthError(e.into()),
                AuthError::UnexpectedError(_) => LoginError::UnexpectedError(e.into()),
            };
            // Use FlashMessage to set flash cookie here instead that setting cookies below
            FlashMessage::error(e.to_string()).send();
            // Using redirect - if auth succeeds, we navigate back to our home page
            let response = HttpResponse::SeeOther()
                .insert_header((LOCATION, "/login"))
                // .insert_header(("Set-Cookie", format!("_flash={e}"))) // there's a dedicated API for cookies just line below
                // .cookie(Cookie::new("_flash", e.to_string())) // we know use FlashMessagesFramework to care about cookies
                .finish();
            Err(InternalError::from_response(e, response))
        }
    }
}

// I anything goes wrong the user will be redirected back to
// the `/login` page with the appropriate error message
fn login_redirect(e: LoginError) -> InternalError<LoginError> {
    FlashMessage::error(e.to_string()).send();
    let response = HttpResponse::SeeOther()
        .insert_header((LOCATION, "/login"))
        .finish();
    InternalError::from_response(e, response)
}

#[derive(thiserror::Error)]
pub enum LoginError {
    #[error("Authentication failed")]
    AuthError(#[source] anyhow::Error),
    #[error("Something went wrong")]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for LoginError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}
