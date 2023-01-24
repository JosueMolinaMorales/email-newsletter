use actix_web::{get, App, HttpServer, Responder};

#[get("/")]
async fn health_check() -> impl Responder {
    "Welcome to the Email Newsletter API v0.0.1-alpha!"
}

pub async fn run() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(health_check)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}