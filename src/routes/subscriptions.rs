use actix_web::{post, HttpResponse, web};
use chrono::Utc;
use sqlx::{query, PgPool};
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

use crate::{domain::{NewSubscriptionForm, SubscriberName, Subscriber, SubscriberEmail}, email_client::EmailClient};


impl TryFrom<NewSubscriptionForm> for Subscriber {
    type Error = String;

    fn try_from(new_sub: NewSubscriptionForm) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(new_sub.name)?;
        let email = SubscriberEmail::parse(new_sub.email)?;

        Ok(Self {
            email,
            name
        })
    }
}

#[tracing::instrument(
    name = "Adding a new subscriber",
    skip(form, pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
#[post("/subscription")]
async fn subscription(
    form: web::Json<NewSubscriptionForm>, 
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>
) -> HttpResponse {
    let new_sub: Subscriber = match form.0.try_into() {
        Ok(sub) => sub,
        Err(_) => return HttpResponse::BadRequest().finish()
    };

    if insert_subscriber(&pool, &new_sub).await.is_err() {
        return HttpResponse::InternalServerError().finish()
    }
    
    // Send an email to the new subscriber
    if email_client
        .send_email(
            new_sub.email, 
            "Welcome!", 
            "Welcome to our newsletter!", 
            "Welcome to our newsletter!"
        ).await.is_err() {
            return HttpResponse::InternalServerError().finish();
    }
    
    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    form: &Subscriber
) -> Result<(), sqlx::Error> {
    query!(
        r#"
        INSERT INTO subscriptions(id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        Uuid::new_v4(),
        form.email.as_ref(),
        form.name.as_ref(),
        Utc::now(),
        "confirmed"
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

pub fn is_valid_name(name: &str) -> bool {
    let is_empty_or_whitespace = name.trim().is_empty();
    let is_too_long = name.graphemes(true).count() > 256;
    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_characters = name.chars().any(|g| forbidden_characters.contains(&g));

    !(is_empty_or_whitespace || is_too_long || contains_forbidden_characters)
}