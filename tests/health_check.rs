use std::net::TcpListener;
use email_newsletter::run;
use serde_json::json;

#[tokio::test]
async fn health_check_test() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/", address))
        .send()
        .await
        .expect("Failted to execute request");

    assert!(res.status().is_success());
    assert!(res.text().await.unwrap() == "Welcome to the Email Newsletter API v0.0.1-alpha!".to_string());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    // Act
    let body = json!({
        "email": "email@email.com",
        "name": "Jake Snow"
    });
    let res = client
        .post(format!("{}/subscription", app_address))
        .body(body.to_string())
        .header("Content-Type", "application/json")
        .send()
        .await
        .expect("Failed to send request");

    // Assert
    assert_eq!(res.status().as_u16(), 200)
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let app_address = spawn_app();
    let client = reqwest::Client::new();

    let test_cases = vec![
        json!({"email": "email@email.com"}).to_string(),
        json!({"name": "name"}).to_string(),
        json!({}).to_string()
    ];
    for body in test_cases {
        let res = client
            .post(format!("{}/subscription", app_address))
            .body(body.clone())
            .header("Content-Type", "application/json")
            .send()
            .await
            .expect("Failed to send request");
        assert_eq!(
            res.status().as_u16(), 
            400,
            "Test Failed for body: {}",
            body
        );
    }
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0")
        .expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let server = run(listener).expect("Failed to bind to address");

    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{}", port)
}