use crate::{routes::*, email_client::EmailClient};
use actix_web::{dev::Server, web::Data, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use std::net::TcpListener;

pub fn run(
    listener: TcpListener, 
    connection_pool: PgPool,
    email_client: EmailClient
) -> Result<Server, std::io::Error> {
    let connection = Data::new(connection_pool);
    let email_client = Data::new(email_client);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(health_check)
            .service(subscription)
            .app_data(connection.clone())
            .app_data(email_client.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
