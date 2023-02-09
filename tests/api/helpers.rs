use email_newsletter::{telemetry::{init_subscriber, get_subscriber}, configuration::{get_configuration, DatabaseSettings}, startup::{get_connection_pool, Application}};
use once_cell::sync::Lazy;
use sqlx::{PgPool, PgConnection, sqlx_macros::migrate, Connection, Executor};
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

impl TestApp {
    pub async fn post_subscription(&self, body: String) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("{}/subscription", &self.address))
            .header("Content-Type", "application/json")
            .body(body)
            .send()
            .await
            .expect("Failed to send request")
    }
}

pub async fn spawn_app() -> TestApp {
    // The first time `initalize` is invoked the code in TRACING is executed
    // All other invocations will instead skip execution
    Lazy::force(&TRACING);

    let configuration = {
        let mut c = get_configuration().expect("Failted to read configuration");
        // Use a different database for each test case
        c.database.database_name = Uuid::new_v4().to_string();
        // use a random OS port
        c.application.port = 0;
        c
    };

    configure_database(&configuration.database).await;

    // Launch the app
    let application = Application::build(configuration.clone())
        .await
        .expect("Failed to create app");
    let address = format!("http://127.0.0.1:{}", application.port());
    let _ = tokio::spawn(application.run_until_stopped());

    TestApp {
        address,
        db_pool: get_connection_pool(&configuration.database),
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
        .expect("Failed to create database");

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

