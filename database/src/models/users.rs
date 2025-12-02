use anyhow::Ok;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Users {
    id: Uuid,
    email: String,
    hashed_password: String,
    display_name: Option<String>,
    created_at: DateTime<Utc>,
}

pub async fn create_user(
    pool: &PgPool,
    email: &str,
    hashed_password: &str,
    display_name: Option<&str>,
) -> anyhow::Result<Uuid> {
    let record: Users = sqlx::query_as!(
        Users,
        r#"
        INSERT INTO users (email , hashed_password , display_name)
        VALUES ($1 , $2 , $3)
        RETURNING id, email, hashed_password, display_name, created_at 
        "#,
        email,
        hashed_password,
        display_name
    )
    .fetch_one(pool)
    .await?;

    Ok(record.id)
}

pub async fn find_by_id_user(pool: &PgPool, id: Uuid) -> anyhow::Result<Users> {
    let record = sqlx::query_as!(
        Users,
        r#"
        SELECT id, email, hashed_password, display_name, created_at
        FROM users
        WHERE id = $1 
        "#,
        id
    )
    .fetch_one(pool)
    .await?;
    Ok(record)
}

pub async fn find_by_email_user(pool: &PgPool, email: &str) -> anyhow::Result<Users> {
    let record = sqlx::query_as!(
        Users,
        r#"
        SELECT id, email, hashed_password, display_name, created_at
        FROM users
        WHERE lower(email) = lower($1) 
        "#,
        email
    )
    .fetch_one(pool)
    .await?;
    Ok(record)
}

pub async fn update_display_name_user(
    pool: &PgPool,
    display_name: Option<&str>,
    id: Uuid,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE users
        SET display_name = $2
        WHERE id = $1
        "#,
        id,
        display_name
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_user(pool: &PgPool, id: Uuid) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM users WHERE id = $1
        "#,
        id
    )
    .execute(pool)
    .await?;
    Ok(())
}
