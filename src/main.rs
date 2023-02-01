use std::net::TcpListener;

use email_newsletter::configuration::get_configuration;
use email_newsletter::startup::run;
use email_newsletter::telemetry::{get_subscriber, init_subscriber};
use sqlx::postgres::PgPoolOptions;

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    // Setting up Logging
    let subscriber = get_subscriber(
        "email-newsletter".into(),
        "info".into(), 
        std::io::stdout
    );
    init_subscriber(subscriber);

    // Panic if we cant read config
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());
    let address = format!("{}:{}", configuration.application.host, configuration.application.port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}
