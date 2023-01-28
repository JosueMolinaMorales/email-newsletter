use actix_web::{get, Responder};


#[get("/")]
async fn health_check() -> impl Responder {
    "Welcome to the Email Newsletter API v0.0.1-alpha!"
}