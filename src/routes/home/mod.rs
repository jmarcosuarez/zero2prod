use actix_web::{http::header::ContentType, HttpResponse};

pub async fn home() -> HttpResponse {
    // We want to read the html file and return it as the body of the GET "/"" endpoint
    // Launch app with cargo and visit: http://localhost:8000 in the browser you should see our newsletter! message
    HttpResponse::Ok()
        // This contentType HTTP header is understood by all HTTP clients (this one is also set on the HTML itself)
        .content_type(ContentType::html())
        .body(include_str!("home.html"))
}
