use crate::routes::*;
use actix_web::{dev::Server, web::Data, App, HttpServer};
use sqlx::PgPool;
use tracing_actix_web::TracingLogger;
use std::net::TcpListener;

pub fn run(listener: TcpListener, connection_pool: PgPool) -> Result<Server, std::io::Error> {
    let connection = Data::new(connection_pool);
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(health_check)
            .service(subscription)
            .app_data(connection.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}
