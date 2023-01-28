use actix_web::{Responder, HttpResponse, post, web};
use serde::{Serialize, Deserialize};


#[derive(Debug, Serialize, Deserialize)]
struct EmailSubscription {
    email: String,
    name: String
}

#[post("/subscription")]
async fn subscription(sub_body: web::Json<EmailSubscription>) -> impl Responder {
    HttpResponse::Ok().json(sub_body.0)
}