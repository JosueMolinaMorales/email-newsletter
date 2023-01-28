use std::net::TcpListener;
use crate::routes::*;
use actix_web::{HttpServer, dev::Server, App};


pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .service(health_check)
            .service(subscription)
    })
    .listen(listener)?
    .run();

    Ok(server)
}