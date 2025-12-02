use anyhow::Ok;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct Action {
    id: Uuid,
    hand_id: Option<Uuid>,
    user_id: Option<Uuid>,
    action_type: String,
    amount: Option<i64>,
    created_at: Option<DateTime<Utc>>,
}

pub async fn insert_action(
    pool: &PgPool,
    hand_id: Option<Uuid>,
    user_id: Option<Uuid>,
    action_type: Option<String>,
    amount: Option<i64>,
) -> anyhow::Result<Uuid> {
    let record = sqlx::query_as!(
        Action,
        r#"
        INSERT INTO actions (hand_id , user_id , action_type , amount)
        VALUES ($1 , $2 , $3 , $4)
        RETURNING id , hand_id ,user_id , action_type, amount,created_at
        "#,
        hand_id,
        user_id,
        action_type,
        amount
    )
    .fetch_one(pool)
    .await?;

    Ok(record.id)
}

pub async fn list_by_hand(pool: &PgPool, hand_id: Option<Uuid>) -> anyhow::Result<Vec<Action>> {
    let records = sqlx::query_as!(
        Action,
        r#"
        SELECT id, hand_id , user_id , action_type , amount, created_at
        FROM actions
        WHERE id = $1
        ORDER BY created_at ASC
        "#,
        hand_id
    )
    .fetch_all(pool)
    .await?;

    Ok(records)
}
