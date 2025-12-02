use anyhow::Ok;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct Hand {
    id: Uuid,
    room_id: Option<Uuid>,
    started_at: Option<DateTime<Utc>>,
    finished_at: Option<DateTime<Utc>>,
    pot: i64,
    board: Option<serde_json::Value>,
    winner_user_id: Option<Uuid>,
    result: Option<serde_json::Value>,
    created_at: DateTime<Utc>,
}

pub async fn create_hand(
    pool: &PgPool,
    room_id: Option<Uuid>,
    started_at: Option<DateTime<Utc>>,
) -> anyhow::Result<Uuid> {
    let record = sqlx::query_as!(
        Hand,
        r#"
        INSERT INTO hands (room_id , started_at)
        VALUES ($1 ,$2)
        RETURNING id , room_id , started_at , finished_at , pot , board , winner_user_id , result , created_at 
        "#,
        room_id,
        started_at
    ).fetch_one(pool).await?;

    Ok(record.id)
}

pub async fn finish_hand(
    pool: &PgPool,
    hand_id: Uuid,
    finished_at: Option<DateTime<Utc>>,
    pot: i64,
    board: Option<serde_json::Value>,
    winner_user_id: Option<Uuid>,
    result: Option<serde_json::Value>,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE hands 
        SET finished_at = $2, pot = $3 , board = $4 , winner_user_id = $5 , result = $6
        WHERE id = $1
        "#,
        hand_id,
        finished_at,
        pot,
        board,
        winner_user_id,
        result
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn find_by_id_hands(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<Hand>> {
    let record = sqlx::query_as!(
        Hand,
        r#"
        SELECT id, room_id, started_at, finished_at, pot, board, winner_user_id, result, created_at
        FROM hands
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;
    Ok(record)
}

pub async fn list_by_hands(
    pool: &PgPool,
    room_id: Option<Uuid>,
    limit: i64,
) -> anyhow::Result<Vec<Hand>> {
    let records = sqlx::query_as!(
        Hand,
        r#"
        SELECT id, room_id, started_at, finished_at, pot, board, winner_user_id, result, created_at
        FROM hands
        WHERE id = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
        room_id,
        limit
    )
    .fetch_all(pool)
    .await?;

    Ok(records)
}
