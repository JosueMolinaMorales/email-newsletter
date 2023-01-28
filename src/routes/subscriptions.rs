use actix_web::{post, web, HttpResponse};
use serde::{Serialize, Deserialize};
use sqlx::{PgPool, query};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize)]
struct EmailSubscription {
    email: String,
    name: String
}

#[post("/subscription")]
async fn subscription(
    form: web::Json<EmailSubscription>,
    pool: web::Data<PgPool>
) -> HttpResponse {
    match query!(
        r#"
        INSERT INTO subscriptions(id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)
        "#,
        Uuid::new_v4(),
        form.0.email,
        form.0.name,
        Utc::now()
    )
    .execute(pool.get_ref())
    .await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}