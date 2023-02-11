
use actix_web::{HttpResponse, get, web};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Parameters {
    subscription_token: String
}

#[tracing::instrument(
    name = "Confrim a pending subscriber"
)]
#[get("/subscriptions/confirm")]
pub async fn confirm(params: web::Query<Parameters>) -> HttpResponse {
    HttpResponse::Ok().finish()
}