use serde_json::json;
use sqlx::query;

use crate::helpers::spawn_app;

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let test_app = spawn_app().await;

    // Act
    let client = reqwest::Client::new();
    let body = json!({
        "email": "email@email.com",
        "name": "Jake Snow"
    });
    let res = client
        .post(format!("{}/subscription", test_app.address))
        .body(body.to_string())
        .header("Content-Type", "application/json")
        .send()
        .await
        .expect("Failed to send request");

    // Assert
    assert_eq!(res.status().as_u16(), 200);

    let saved = query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "email@email.com");
    assert_eq!(saved.name, "Jake Snow")
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        (json!({"email": "email@email.com"}).to_string(), "No Name"),
        (json!({"name": "name"}).to_string(), "No email"),
        (json!({}).to_string(), "No body"),
    ];
    for (body, description) in test_cases {
        let res = client
            .post(format!("{}/subscription", test_app.address))
            .body(body.clone())
            .header("Content-Type", "application/json")
            .send()
            .await
            .expect("Failed to send request");
        assert_eq!(res.status().as_u16(), 400, "Test Failed for: {}", description);
    }
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_invalid() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let test_cases = vec![
        (json!({"email": "notanemail", "name": "name"}).to_string(), "Not an email"),
        (json!({"name": "", "email": "email@email.com"}).to_string(), "Name empty"),
        (json!({"name": "name", "email": ""}).to_string(), "Email Empty")
    ];

    for (body, description) in test_cases {
        let res = client
            .post(format!("{}/subscription", test_app.address))
            .body(body.clone())
            .header("Content-Type", "application/json")
            .send()
            .await
            .expect("Failed to send request");
        assert_eq!(res.status().as_u16(), 400, "Test Failed for: {}", description);
    }
}