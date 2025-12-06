use anyhow::Ok;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct RefreshToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub revoked: bool,
    pub created_at: DateTime<Utc>,
}

pub async fn insert_tokens(
    pool: &PgPool,
    user_id: Uuid,
    token_hash: &str,
    expires_at: DateTime<Utc>,
) -> anyhow::Result<Uuid> {
    let record = sqlx::query_as!(
        RefreshToken,
        r#"
        INSERT INTO refresh_tokens (user_id , token_hash , expires_at)
        VALUES ($1 , $2 , $3)
        RETURNING id, user_id, token_hash, expires_at, revoked, created_at
        "#,
        user_id,
        token_hash,
        expires_at
    )
    .fetch_one(pool)
    .await?;

    Ok(record.id)
}

pub async fn find_by_hash_tokens(
    pool: &PgPool,
    token_hash: &str,
) -> anyhow::Result<Option<RefreshToken>> {
    let record = sqlx::query_as!(
        RefreshToken,
        r#"
        SELECT id, user_id, token_hash, expires_at, revoked, created_at
        FROM refresh_tokens 
        WHERE token_hash = $1
        "#,
        token_hash
    )
    .fetch_optional(pool)
    .await?;

    Ok(record)
}

pub async fn revoke(pool: &PgPool, id: Uuid) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE refresh_tokens SET revoked = true WHERE id = $1
        "#,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn revoke_all_for_user(pool: &PgPool, user_id: Uuid) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE refresh_tokens SET revoked = true WHERE user_id = $1
        "#,
        user_id
    )
    .execute(pool)
    .await?;

    Ok(())
}
