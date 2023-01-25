use std::net::TcpListener;

use actix_web::{get, App, HttpServer, Responder, dev::Server};

#[get("/")]
async fn health_check() -> impl Responder {
    "Welcome to the Email Newsletter API v0.0.1-alpha!"
}

pub fn run(listener: TcpListener) -> Result<Server, std::io::Error> {
    let server = HttpServer::new(|| {
        App::new()
            .service(health_check)
    })
    .listen(listener)?
    .run();

    Ok(server)
}