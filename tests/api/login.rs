use crate::helpers::{spawn_app, assert_is_redirect_to};
use std::collections::HashSet;


#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    // Arrange
    let app = spawn_app().await;

    // Act 1 - Try to login
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });
    let response = app.post_login(&login_body).await;

    // Assert
    assert_is_redirect_to(&response, "/login");

    let cookies: HashSet<_> = response
    .headers()
    .get_all("Set-Cookie")
    .into_iter()
    .collect();

    let flash_cookie = response.cookies().find(|c| c.name() == "_flash").unwrap();

    // Act 2 - Follow the redirect
    let html_page = app.get_login_html().await;
    
    // Assert 2
    assert!(html_page.contains(r#"<p><i>Authentication failed</i></p>"#));

    // Act 3 - Reload the login page
    let html_page = app.get_login_html().await;
    assert!(!html_page.contains("Authentication failed"));
}
