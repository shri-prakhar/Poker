use anyhow::Ok;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct Rooms {
    pub id: Uuid,
    pub room_name: Option<String>,
    pub host_user_id: Option<Uuid>,
    pub room_status: String,
    pub max_players: Option<i16>,
    pub created_at: DateTime<Utc>,
}

pub async fn create_rooms(
    pool: &PgPool,
    room_name: Option<String>,
    host_user_id: Option<Uuid>,
    max_players: Option<i16>,
) -> anyhow::Result<Uuid> {
    let record = sqlx::query_as!(
        Rooms,
        r#"
        INSERT INTO rooms (room_name , host_user_id , max_players)
        VALUES ($1 , $2 , $3)
        RETURNING id , room_name , host_user_id , room_status , max_players , created_at
        "#,
        room_name,
        host_user_id,
        max_players
    )
    .fetch_one(pool)
    .await?;
    Ok(record.id)
}

pub async fn find_by_id_rooms(pool: &PgPool, id: Uuid) -> anyhow::Result<Option<Rooms>> {
    let record = sqlx::query_as!(
        Rooms,
        r#"
        SELECT id , room_name , host_user_id , room_status , max_players , created_at
        FROM rooms
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await?;

    Ok(record)
}

pub async fn list_by_status(pool: &PgPool, room_status: &str) -> anyhow::Result<Vec<Rooms>> {
    let record = sqlx::query_as!(
        Rooms,
        r#"
        SELECT id , room_name , host_user_id , room_status , max_players , created_at
        FROM rooms
        WHERE room_status = $1
        ORDER BY created_at DESC
        "#,
        room_status
    )
    .fetch_all(pool)
    .await?;

    Ok(record)
}

pub async fn update_rooms(pool: &PgPool, id: Uuid, room_status: &str) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE rooms SET room_status = $1 
        WHERE id = $2
        "#,
        room_status,
        id
    )
    .execute(pool)
    .await?;

    Ok(())
}
