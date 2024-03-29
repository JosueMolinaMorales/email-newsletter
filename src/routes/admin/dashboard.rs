use actix_session::Session;
use actix_web::{get, web, HttpResponse};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

fn e500<T>(e: T) -> actix_web::Error
where
    T: std::fmt::Debug + std::fmt::Display + 'static,
{
    actix_web::error::ErrorInternalServerError(e)
}

#[get("/admin/dashboard")]
pub async fn admin_dashboard(
    session: Session,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let username = if let Some(user_id) = session.get::<Uuid>("user_id").map_err(e500)? {
        get_username(user_id, &pool).await.map_err(e500)?
    } else {
        todo!()
    };
    Ok(HttpResponse::Ok().body(format!(
        r#"
    <!DOCTYPE html>
    <html lang="en">
        <head>
            <meta http-equiv="content-type" content="text/html"; charset="utf-8" />
            <title>Admin Dashboard</title>
        </head>
        <body>
            <h1>Admin Dashboard</h1>
            <p>Hello, {username}!</p>
        </body>
    "#
    )))
}

async fn get_username(user_id: Uuid, pool: &PgPool) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT username
        FROM users
        WHERE user_id = $1
        "#,
        user_id
    )
    .fetch_one(pool)
    .await
    .context("Failed to perform a query to retrieve a username.")?;
    Ok(row.username)
}
