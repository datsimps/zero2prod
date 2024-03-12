use crate::helpers::{spawn_app, assert_is_redirect_to};

/*
#[tokio::test]
async fn redirect_to_admin_dashboard_after_login_success(){
    // Arrange
    let app = spawn_app().await;

    // Act 1 - Part 1 - Login
    let login_body = serde_json::json!({
        "username": "random-username",
        "password": "random-password"
    });
    let response = app.post_login(&login_body).await;

    // Assert
    assert_is_redirect_to(&response, "/login");

    // Act - Part 2 - Follow the redirect
    let html_page = app.get_admin_dashboard_html().await;
   
    
    println!("html page: {}", &html_page);
    // Assert 2
    assert!(html_page.contains(&format!("Weclome {}", app.test_user.username)));

}
*/

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

    // Act 2 - Follow the redirect
    let html_page = app.get_login_html().await;
    println!("html page {}", &html_page);
    // Assert 2
    assert!(html_page.contains("<p><i>Authenicaton failed</i></p>"));

    // Act 3 - Reload the login page
    let html_page = app.get_login_html().await;
    assert!(!html_page.contains("Authenicaton failed"));
}
