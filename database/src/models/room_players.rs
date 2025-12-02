use anyhow::Ok;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct RoomPlayers {
    room_id: Uuid,
    seat: i16,
    user_id: Uuid,
    chips: i64,
    connected: Option<bool>,
    is_dealer: Option<bool>,
}

//add a player to seat
pub async fn add_player(
    pool: &PgPool,
    room_id: Uuid,
    seat: i16,
    user_id: Uuid,
    chips: i64,
    is_dealer: bool,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO room_players (room_id , seat , user_id , chips, connected,is_dealer)
        VALUES ($1 , $2 , $3 , $4 , true , $5)
        ON CONFLICT (room_id , seat) DO UPDATE
        SET user_id = EXCLUDED.user_id, chips = EXCLUDED.chips , connected = true , is_dealer = EXCLUDED.is_dealer
        "#,
        room_id,
        seat,
        user_id,
        chips,
        is_dealer
    ).execute(pool).await?;

    Ok(())
}

pub async fn remove_players(pool: &PgPool, room_id: Uuid, seat: i16) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        DELETE FROM room_players WHERE room_id = $1 AND seat = $2
        "#,
        room_id,
        seat
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn list_by_room(pool: &PgPool, room_id: Uuid) -> anyhow::Result<Vec<RoomPlayers>> {
    let records = sqlx::query_as!(
        RoomPlayers,
        r#"
        SELECT room_id , seat , user_id , chips , connected, is_dealer
        FROM room_players
        WHERE room_id = $1
        ORDER BY seat
        "#,
        room_id
    )
    .fetch_all(pool)
    .await?;

    Ok(records)
}

pub async fn update_chips(
    pool: &PgPool,
    room_id: Uuid,
    chips: i64,
    seat: i16,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE room_players SET chips = $3 
        WHERE room_id = $1 AND seat = $2   
        "#,
        room_id,
        seat,
        chips
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn set_connected(
    pool: &PgPool,
    room_id: Uuid,
    seat: i16,
    connected: bool,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        UPDATE room_players SET connected = $3 
        WHERE room_id = $1 AND seat = $2
        "#,
        room_id,
        seat,
        connected
    )
    .execute(pool)
    .await?;

    Ok(())
}
