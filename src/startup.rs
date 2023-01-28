use std::net::TcpListener;
use crate::routes::*;
use actix_web::{HttpServer, dev::Server, App, web::Data};
use sqlx::PgPool;


pub fn run(
    listener: TcpListener,
    connection_pool: PgPool
) -> Result<Server, std::io::Error> {
    let connection = Data::new(connection_pool);
    let server = HttpServer::new(move || {
        App::new()
            .service(health_check)
            .service(subscription)
            .app_data(connection.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}