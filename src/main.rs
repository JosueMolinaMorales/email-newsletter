use std::net::TcpListener;

use email_newsletter::startup::run;
use email_newsletter::configuration::get_configuration;
use sqlx::PgPool;


#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    // Panic if we cant read config
    let configuration = get_configuration().expect("Failed to read configuration");
    let connection_pool = PgPool::connect(
        &configuration.database.connection_string()
        )
        .await
        .expect("Cannot connect to Postgres");
    let address = format!("127.0.0.1:{}", configuration.application_port);
    let listener = TcpListener::bind(address)?;
    run(listener, connection_pool)?.await
}