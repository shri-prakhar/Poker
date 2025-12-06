use anyhow::Ok;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, prelude::FromRow};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize, FromRow)]
pub struct HandPlayers {
    pub hand_id: Option<Uuid>,
    pub seat: i16,
    pub user_id: Option<Uuid>,
    pub hole_cards: Option<serde_json::Value>,
    pub chips_before: Option<i64>,
    pub chips_after: Option<i64>,
}

pub async fn insert_player(
    pool: &PgPool,
    hand_id: Option<Uuid>,
    seat: i16,
    user_id: Option<Uuid>,
    hole_cards: Option<serde_json::Value>,
    chips_before: Option<i64>,
    chips_after: Option<i64>,
) -> anyhow::Result<()> {
    sqlx::query!(
            r#"
            INSERT INTO hand_players (hand_id , seat , user_id , hole_cards , chips_before, chips_after)
            VALUES ($1 ,$2 , $3 , $4 , $5 , $6)
            "#,
            hand_id,
            seat,
            user_id,
            hole_cards,
            chips_before,
            chips_after
        ).execute(pool).await?;

    Ok(())
}

pub async fn list_by_hand_players(
    pool: &PgPool,
    hand_id: Option<Uuid>,
) -> anyhow::Result<Vec<HandPlayers>> {
    let records = sqlx::query_as!(
        HandPlayers,
        r#"
            SELECT hand_id, seat, user_id, hole_cards, chips_before, chips_after
            FROM hand_players
            WHERE hand_id = $1
            ORDER BY seat
            "#,
        hand_id
    )
    .fetch_all(pool)
    .await?;

    Ok(records)
}
