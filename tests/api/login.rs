use crate::helpers::{assert_is_redirect_to, spawn_app};

// Flash messages:
//   where the user/API exchange all related information via a side-channel (cookies)
//   invisible to the browser history. The  error message should be ephemeral - the cookie
//   is consumed when the error message is rendered.. If the page is reloaded, the error
//   message should not be shown again
#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    // Arrange
    let app = spawn_app().await;

    // Act
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password",
    });
    let response = app.post_login(&login_body).await;

    // NOTE: No longer asserting facts related to cookies

    // let cookies: HashSet<_> = response
    //     .headers()
    //     .get_all("Set-Cookie")
    //     .into_iter()
    //     .collect();

    // Assert
    assert_is_redirect_to(&response, "/login");
    // assert!(cookies.contains(&HeaderValue::from_str("_flash=Authentication failed").unwrap()));

    // Assert (with reqwest cookie feature flag)
    // let flash_cookie = response.cookies().find(|c| c.name() == "_flash").unwrap();
    // assert_eq!(flash_cookie.value(), "Authentication failed");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_login_html().await;
    assert!(html_page.contains("Authentication failed"));

    // Act - Part 3 - Reload the login page
    let html_page = app.get_login_html().await;
    assert!(!html_page.contains("Authentication failed"));
}

#[tokio::test]
async fn redirect_to_admin_dashboard_after_login_success() {
    // Arrange
    let app = spawn_app().await;

    // Act - Part 1 - Login
    let login_body = serde_json::json!({
    "username": &app.test_user.username,
    "password": &app.test_user.password,
    });
    let response = app.post_login(&login_body).await;
    assert_is_redirect_to(&response, "/admin/dashboard");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", app.test_user.username)));
}
