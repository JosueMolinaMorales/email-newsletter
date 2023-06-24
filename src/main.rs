use email_newsletter::configuration::get_configuration;
use email_newsletter::startup::Application;
use email_newsletter::telemetry::{get_subscriber, init_subscriber};

#[actix_web::main]
async fn main() -> Result<(), std::io::Error> {
    // Setting up Logging
    let subscriber = get_subscriber("email-newsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Panic if we cant read config
    let configuration = get_configuration().expect("Failed to read configuration");
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}
