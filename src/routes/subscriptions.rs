use actix_web::{post, HttpResponse, web};
use chrono::Utc;
use sqlx::{query, PgPool};
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
    if send_confirmation_email(&email_client, new_sub).await.is_err() {
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
        "pending_confirmation"
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_sub)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_sub: Subscriber
) -> Result<(), reqwest::Error> {
    let confirmation_link = "https://localhost/subscriptions/confirm";
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    let html_body = format!(
        "Welcome to our newsletter! <br/>
        Click <a href=\"{}\">here</a> to confirm your subscription
        ",
        confirmation_link
    );
    email_client
        .send_email(
            new_sub.email, 
            "Welcome!", 
            &html_body, 
            &plain_body
        ).await
}