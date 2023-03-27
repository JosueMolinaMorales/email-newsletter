use wiremock::{Mock, matchers::{any, path, method}, ResponseTemplate};

use crate::helpers::{spawn_app, TestApp};


#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let app = spawn_app().await;
    create_unconfirmed_subscriber(&app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&app.email_server)
        .await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter Title",
        "content": {
            "text": "Newsletter body as plain text",
            "html": "<p>Newsletter body as HTML</p>"
        }
    });
    let response = reqwest::Client::new()
        .post(&format!("{}/newsletters", &app.address))
        .json(&newsletter_request_body)
        .send()
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 200);
}

async fn create_unconfirmed_subscriber(app: &TestApp) {
    let body = serde_json::json!({
        "email": "email@email.com",
        "name": "Name"
    });
    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfrimed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    
    app.post_subscription(body.to_string())
        .await
        .error_for_status()
        .unwrap();
}