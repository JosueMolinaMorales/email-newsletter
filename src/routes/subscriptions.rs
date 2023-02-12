use actix_web::{post, HttpResponse, web};
use chrono::Utc;
use rand::{thread_rng, Rng, distributions::Alphanumeric};
use sqlx::{query, PgPool};
use uuid::Uuid;

use crate::{domain::{NewSubscriptionForm, SubscriberName, Subscriber, SubscriberEmail}, email_client::EmailClient, startup::{ApplicationBaseUrl, ApplicationPort}};


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
    skip(form, pool, email_client, base_url),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
#[post("/subscription")]
async fn subscription(
    form: web::Json<NewSubscriptionForm>, 
    pool: web::Data<PgPool>,
    email_client: web::Data<EmailClient>,
    base_url: web::Data<ApplicationBaseUrl>,
    port: web::Data<ApplicationPort>
) -> HttpResponse {
    let new_sub: Subscriber = match form.0.try_into() {
        Ok(sub) => sub,
        Err(_) => return HttpResponse::BadRequest().finish()
    };

    let subscriber_id = match insert_subscriber(&pool, &new_sub).await {
        Ok(subscriber_id) => subscriber_id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    let subscription_token = generate_subscription_token();
    if store_token(&pool, subscriber_id, &subscription_token)
        .await
        .is_err() 
    {
        return HttpResponse::InternalServerError().finish();
    }
    // Send an email to the new subscriber
    if send_confirmation_email(
        &email_client,
        new_sub,
        &base_url.0,
        port.0,
        &subscription_token
    )
    .await
    .is_err() 
    {
        return HttpResponse::InternalServerError().finish();
    }
    
    HttpResponse::Ok().finish()
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, pool)
)]
pub async fn store_token(
    pool: &PgPool,
    subscriber_id: Uuid,
    subscription_token: &str
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
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
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    form: &Subscriber
) -> Result<Uuid, sqlx::Error> {
    let subscriber_id = Uuid::new_v4();
    query!(
        r#"
        INSERT INTO subscriptions(id, email, name, subscribed_at, status)
        VALUES ($1, $2, $3, $4, $5)
        "#,
        subscriber_id,
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
    Ok(subscriber_id)
}

#[tracing::instrument(
    name = "Send a confirmation email to a new subscriber",
    skip(email_client, new_sub, base_url, subscription_token)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_sub: Subscriber,
    base_url: &str,
    port: u16,
    subscription_token: &str,
) -> Result<(), reqwest::Error> {
    let confirmation_link = format!(
        "{}:{}/subscriptions/confirm?subscription_token={}",
        port,
        base_url,
        subscription_token
    );
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

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}