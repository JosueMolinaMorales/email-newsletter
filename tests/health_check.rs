use email_newsletter::{
    configuration::{get_configuration, DatabaseSettings},
    startup::run, telemetry::{get_subscriber, init_subscriber}, email_client::EmailClient,
};
use once_cell::sync::Lazy;
use serde_json::json;
use sqlx::{migrate, query, Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(
            subscriber_name, default_filter_level, std::io::stdout
        );
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::sink
        );
        init_subscriber(subscriber);
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

#[tokio::test]
async fn health_check_test() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let res = client
        .get(format!("{}/", test_app.address))
        .send()
        .await
        .expect("Failted to execute request");

    assert!(res.status().is_success());
    assert!(
        res.text().await.unwrap()
            == "Welcome to the Email Newsletter API v0.0.1-alpha!".to_string()
    );
}

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

async fn spawn_app() -> TestApp {
    // The first time `initalize` is invoked the code in TRACING is executed
    // All other invocations will instead skip execution
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    let mut configuration = get_configuration().expect("Failed to read configuration file");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    // Build a new email client
    let sender_email = configuration.email_client.sender().expect("Invalid email address");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );
    let server = run(listener, connection_pool.clone(), email_client).expect("Failed to bind to address");

    let _ = tokio::spawn(server);

    TestApp {
        address,
        db_pool: connection_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    // Create Database
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connection to postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Faile to create database");

    // Migrate Database
    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to postgres");
    migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
