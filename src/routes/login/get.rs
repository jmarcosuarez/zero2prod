use actix_web::{http::header::ContentType, HttpResponse};
use actix_web_flash_messages::IncomingFlashMessages;
use std::fmt::Write;

pub async fn login_form(flash_messages: IncomingFlashMessages) -> HttpResponse {
    let mut error_html = String::new();

    // Accommodates multiple flash messages. No need to deal with the cookie API, neither to retrieve incoming
    // flash messages nor to make sure they are erased after having been read. The validity of out cookie signature is
    // verified as well, before the request handler is invoked.
    for m in flash_messages.iter() {
        writeln!(error_html, "<p><i>{}</p></i>", m.content()).unwrap();
    }

    HttpResponse::Ok()
        .content_type(ContentType::html())
        // We set flash cookie on the response to `Max-Age=0` to remove the flash messages stored in the user's browser.
        // .cookie(
        //     Cookie::build("_flash", "")
        //         .max_age(actix_web::cookie::time::Duration::ZERO)
        //         .finish(),
        // ) // this is done more clearly below
        .body(format!(
            r#"<!DOCTYPE html>
            <html lang="en">
            <head>
            <meta http-equiv="content-type" content="text/html; charset=utf-8">
            <title>Login</title>
            </head>
            <body>
            {error_html}
            <form action="/login" method="post">
                <label>Username
                    <input
                        type="text"
                        placeholder="Enter Username"
                        name="username"
                    >
                </label>
                <label>Password
                    <input
                        type="password"
                        placeholder="Enter Password"
                        name="password"
                    >
                </label>
                <button type="submit">Login</button>
            </form>
            </body>
            </html>"#,
        ))
    // No more cookie removal - using FlashMessage instead
    // Use `add_removal_cookie()` to clear the flash cookie
    // response
    //     .add_removal_cookie(&Cookie::new("_flash", ""))
    //     .unwrap();

    // response
}
