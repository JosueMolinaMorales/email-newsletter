use serde_json::{json, Value};
use sqlx::query;
use wiremock::{Mock, matchers::{method, path}, ResponseTemplate};

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let test_app = spawn_app().await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;
    // Act
    let body = json!({
        "email": "email@email.com",
        "name": "Jake Snow"
    });
    let res = test_app.post_subscription(body.to_string()).await;

    // Assert
    assert_eq!(res.status().as_u16(), 200);
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    let app = spawn_app().await;
    let body = json!({
        "email": "email@email.com",
        "name": "Name"
    });
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;
    
    app.post_subscription(body.to_string()).await;
    
    let saved = query!("SELECT email, name, status FROM subscriptions")
        .fetch_one(&app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "email@email.com");
    assert_eq!(saved.name, "Name");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        (json!({"email": "email@email.com"}).to_string(), "No Name"),
        (json!({"name": "name"}).to_string(), "No email"),
        (json!({}).to_string(), "No body"),
    ];
    for (body, description) in test_cases {
        let res = test_app.post_subscription(body).await; 
        assert_eq!(res.status().as_u16(), 400, "Test Failed for: {}", description);
    }
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_invalid() {
    let test_app = spawn_app().await;

    let test_cases = vec![
        (json!({"email": "notanemail", "name": "name"}).to_string(), "Not an email"),
        (json!({"name": "", "email": "email@email.com"}).to_string(), "Name empty"),
        (json!({"name": "name", "email": ""}).to_string(), "Email Empty")
    ];

    for (body, description) in test_cases {
        let res = test_app.post_subscription(body).await; 
        assert_eq!(res.status().as_u16(), 400, "Test Failed for: {}", description);
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let app = spawn_app().await;
    let body = json!({
        "email": "email@email.com",
        "name": "Jake Snow"
    });

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&app.email_server)
        .await;

    app.post_subscription(body.to_string()).await;
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_with_a_link() {
    // Arrange
    let app = spawn_app().await;
    let body = json!({
        "email": "email@email.com",
        "name": "Name"
    });

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&app.email_server)
        .await;

    app.post_subscription(body.to_string()).await;

    // Assert
    let email_request = &app.email_server.received_requests().await.unwrap()[0];
    let body: Value = serde_json::from_slice(&email_request.body).unwrap();

    let get_link = |s: &str| {
        let links: Vec<_> = linkify::LinkFinder::new()
            .links(s)
            .filter(|l| *l.kind() == linkify::LinkKind::Url)
            .collect();
        assert_eq!(links.len(), 1);
        links[0].as_str().to_owned()
    };

    let html_link = get_link(&body["HtmlBody"].as_str().unwrap());
    let text_link = get_link(&body["TextBody"].as_str().unwrap());

    assert_eq!(html_link, text_link)
}