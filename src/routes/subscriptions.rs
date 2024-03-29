use std::fmt::Display;

use actix_web::{post, web, HttpResponse, ResponseError};
use anyhow::{anyhow, Context};
use chrono::Utc;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use reqwest::StatusCode;
use sqlx::{query, PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    domain::{NewSubscriptionForm, Subscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    startup::{ApplicationBaseUrl, ApplicationPort},
};

impl TryFrom<NewSubscriptionForm> for Subscriber {
    type Error = String;

    fn try_from(new_sub: NewSubscriptionForm) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(new_sub.name)?;
        let email = SubscriberEmail::parse(new_sub.email)?;

        Ok(Self { email, name })
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
    port: web::Data<ApplicationPort>,
) -> Result<HttpResponse, SubscribeError> {
    let new_sub: Subscriber = form.0.try_into().map_err(SubscribeError::ValidationError)?;
    let mut transaction = pool
        .begin()
        .await
        .context("An unexpected error occurred while trying to begin a transaction.")?;

    let subscriber_id = insert_subscriber(&mut transaction, &new_sub)
        .await
        .context("An unexpected error occurred while trying to insert a new subscriber.")?;

    let subscription_token = generate_subscription_token();

    store_token(&mut transaction, subscriber_id, &subscription_token)
        .await
        .context("Failed to store the confirmation token for a new subcriber.")?;

    transaction
        .commit()
        .await
        .context("An unexpected error occurred while trying to commit the transaction.")?;

    // Send an email to the new subscriber
    send_confirmation_email(
        &email_client,
        new_sub,
        &base_url.0,
        port.0,
        &subscription_token,
    )
    .await
    .context("Failed to send a confirmation email.")?;

    Ok(HttpResponse::Ok().finish())
}

#[tracing::instrument(
    name = "Store subscription token in the database",
    skip(subscription_token, transaction)
)]
pub async fn store_token(
    transaction: &mut Transaction<'_, Postgres>,
    subscriber_id: Uuid,
    subscription_token: &str,
) -> Result<(), StoreTokenError> {
    sqlx::query!(
        r#"INSERT INTO subscription_tokens (subscription_token, subscriber_id)
        VALUES ($1, $2)"#,
        subscription_token,
        subscriber_id
    )
    .execute(transaction)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        StoreTokenError(e)
    })?;
    Ok(())
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, transaction)
)]
pub async fn insert_subscriber(
    transaction: &mut Transaction<'_, Postgres>,
    form: &Subscriber,
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
    .execute(transaction)
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
) -> Result<(), SubscribeError> {
    let confirmation_link = format!(
        "{}:{}/subscriptions/confirm?subscription_token={}",
        port, base_url, subscription_token
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
        .send_email(&new_sub.email, "Welcome!", &html_body, &plain_body)
        .await
        .map_err(|e| SubscribeError::UnexpectedError(anyhow!(e)))
}

fn generate_subscription_token() -> String {
    let mut rng = thread_rng();
    std::iter::repeat_with(|| rng.sample(Alphanumeric))
        .map(char::from)
        .take(25)
        .collect()
}

pub struct StoreTokenError(sqlx::Error);

impl Display for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "A database error was encountered trying to store a subscription token."
        )
    }
}

impl std::error::Error for StoreTokenError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

impl std::fmt::Debug for StoreTokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

pub fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    write!(f, "{}\n", e)?;
    let mut source = e.source();
    while let Some(e) = source {
        write!(f, "Caused by: \n\t{}", e)?;
        source = e.source();
    }
    Ok(())
}

#[derive(thiserror::Error)]
pub enum SubscribeError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl std::fmt::Debug for SubscribeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

impl ResponseError for SubscribeError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::ValidationError(_) => StatusCode::BAD_REQUEST,
            Self::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
