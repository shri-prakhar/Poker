use anyhow::Ok;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct UserSessions {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_seen: Option<DateTime<Utc>>,
}

pub async fn create_user_sessions(
    pool: &PgPool,
    user_id: Uuid,
    device_name: Option<&str>,
) -> anyhow::Result<Uuid> {
    let record = sqlx::query_as!(
        UserSessions,
        r#"
        INSERT INTO user_sessions ( device_name , user_id ) 
        VALUES ($1 , $2)
        RETURNING id , user_id , device_name, created_at, last_seen 
        "#,
        device_name,
        user_id
    )
    .fetch_one(pool)
    .await?;

    Ok(record.id)
}

pub async fn find_by_id_user_sessions(
    pool: &PgPool,
    id: Uuid,
) -> anyhow::Result<Option<UserSessions>> {
    let record = sqlx::query_as!(
        UserSessions,
        r#"
        SELECT id , user_id , device_name, created_at, last_seen
        FROM user_sessions
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(record)
}

pub async fn list_by_user_user_sessions(
    pool: &PgPool,
    user_id: Uuid,
) -> anyhow::Result<Vec<UserSessions>> {
    let record = sqlx::query_as!(
        UserSessions,
        r#"
        SELECT id , user_id , device_name, created_at, last_seen
        FROM user_sessions
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
        user_id
    )
    .fetch_all(pool)
    .await?;

    Ok(record)
}

pub async fn touch_last_seen_user_sessions(
    pool: &PgPool,
    id: Uuid,
    ts: DateTime<Utc>,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE user_sessions SET last_seen = $2  WHERE id = $1
        "#,
        id,
        ts
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_user_sessions(pool: &PgPool, id: Uuid) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM user_sessions WHERE id = $1
        "#,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}
