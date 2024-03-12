use actix_web::{HttpResponse, web};
use uuid::Uuid;
use actix_web::http::header::ContentType;
use anyhow::Context;
use sqlx::PgPool;
use crate::session_state::TypedSession;
use actix_web::http::header::LOCATION;
use crate::utils::e500;


pub async fn admin_dashboard(
    session: TypedSession,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let username = if let Some(user_id) = session
        .get_user_id()
        .map_err(e500)?
    {
        get_username(user_id, &pool).await.map_err(e500)?
    } else { 
        return Ok(HttpResponse::SeeOther()
            .insert_header((LOCATION, "/login"))
            .finish());
            };

    Ok(HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(format!(
                r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta http-equiv="content-type" content="text/html; charset=utf-8">
<title>Admin Dashboard</title>
</head>
<body>
    <p>Welcome {username}!</p>
    <p>Available actions:</p>
    <ol>
        <li><a href="/admin/password">Change password</a></li>
    </ol>
</body>
</html>"#
        )))
}
#[tracing::instrument(name = "Get username", skip(pool))]
pub async fn get_username(
    user_id: Uuid,
    pool: &PgPool,
) -> Result<String, anyhow::Error> {
    let row = sqlx::query!(
        r#"
        SELECT username
        FROM users
        WHERE user_id = $1
        "#,
        user_id,
    )
        .fetch_one(pool)
        .await
        .context("Failed to perform a query to retrieve a username.")?;
    Ok(row.username)

}
