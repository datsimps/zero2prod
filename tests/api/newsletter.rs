use crate::helpers::{spawn_app, TestApp, ConfirmationLinks, assert_is_redirect_to};
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};
use std::time::Duration;

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks{
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(&email_request)
}


async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await.html;
    reqwest::get(confirmation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    // Arrange
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        // no request is fired at Postmark
        .expect(0)
        .mount(&app.email_server)
        .await;

    // Act
    //   test newsletter
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let response = app.post_publish_newsletter(&newsletter_request_body).await;

    // Assert
    assert_is_redirect_to(&response, "/admin/newsletters");
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    // Arrange
    //  create the app and user then login
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act test newsletter
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    let html_page = app.get_publish_newsletter_html().await;
    println!("html page {:?}", html_page);
    println!("response: {}", &response.status().as_u16());
    // Assert
    assert_is_redirect_to(&response, "/admin/newsletters");
}

#[tokio::test]
pub async fn newsletter_creation_is_idempotent() {
    // Arrange 
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("post"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - submit newsletter form 
    let newsletter_request_body = serde_json::json!({
            "title": "newsletter title",
            "text_content": "newsletter body as plain text",
            "html_content": "<p>newsletter body as html</p>",
            "idempotency_key": uuid::Uuid::new_v4().to_string()
        });

    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 2 - Follow the redirect 
    let html_page = app.get_publish_newsletter_html().await;
    assert!(
        html_page.contains("The newsletter issue has been published!")
    );

    // Act - Part 3 - Submit newsletter form **again**
    let response = app.post_publish_newsletter(&newsletter_request_body).await;
    assert_is_redirect_to(&response, "/admin/newsletters");

    // Act - Part 4 - Follow the redirect 

    let html_page = app.get_publish_newsletter_html().await;
    println!("{}", &html_page);
    assert!(
        html_page.contains("The newsletter issue has been published!")
    );

    // Mock verifies on drop that we sent the newsletter email ONCE

}

#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() {
    // Arrange 
    let app = spawn_app().await;
    create_confirmed_subscriber(&app).await;
    app.test_user.login(&app).await;

    Mock::given(path("/email"))
        .and(method("post"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&app.email_server)
        .await;

    // Act - Part 1 - submit newsletter form 
    let newsletter_request_body = serde_json::json!({
            "title": "newsletter title",
            "text_content": "newsletter body as plain text",
            "html_content": "<p>newsletter body as html</p>",
            "idempotency_key": uuid::Uuid::new_v4().to_string()
        });

    let response1 = app.post_publish_newsletter(&newsletter_request_body);
    let response2 = app.post_publish_newsletter(&newsletter_request_body);
    let (response1, response2) = tokio::join!(response1, response2);

    assert_eq!(response1.status(), response2.status());
    assert_eq!(
        response1.text().await.unwrap(),
        response2.text().await.unwrap()
    );

    // Mock verifies on drop that we only sent one email
}


