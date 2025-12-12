use std::sync::{Arc, RwLock};

use dashmap::DashMap;
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{config::Setting, poker_engine::Card, ws_server::ClientInfo};

#[derive(Debug , Clone , Serialize , Deserialize)]
pub struct OutgoingEvent{
  pub event_type:String,
  pub room_id : String,
  pub payload : serde_json::Value,
  pub emitted_at : i64
}
#[derive(Debug, Clone)]
pub struct PlayerSlot {
    pub user_id: Uuid,
    pub seat: u8,
    pub chips: i64,
    pub connected: bool,
}

#[derive(Debug, Clone)]
pub struct HandState {
    pub id: Uuid,
    pub started_at: i64,
    pub pot: i64,
    pub board: Vec<Card>,              //flop . turn , river cards
    pub hole_cards: Vec<(Card, Card)>, //per seat
    pub current_turn: Option<usize>,
    pub round: String, //{current stage :"pre-flop" , "flop" , "turn" , "river"}
    pub players_in_hand: Vec<bool>, //false means player folded
}

#[derive(Debug)]
pub struct RoomState {
    pub room_id: String,
    pub max_players: usize,
    pub seats: Vec<Option<PlayerSlot>>,
    pub dealer_index: Option<usize>,
    pub active_hand: Option<HandState>,
    pub turn_task: Option<CancellationToken>,
}
impl RoomState {
    pub fn new(room_id: &str, max_players: usize) -> Self {
        RoomState {
            room_id: room_id.to_string(),
            max_players,
            seats: vec![None; max_players],
            dealer_index: None,
            active_hand: None,
            turn_task: None,
        }
    }
}
#[derive(Clone)]
pub struct GameManager {
    pub rooms: Arc<DashMap<String, Arc<RwLock<RoomState>>>>,
    pub pool: PgPool,
    pub redis: ConnectionManager,
    pub client_registry: Arc<DashMap<String, Vec<ClientInfo>>>,
    pub setting: Setting,
}

impl GameManager {
    pub async fn new(
        pool: PgPool,
        redis: ConnectionManager,
        client_registry: Arc<DashMap<String, Vec<ClientInfo>>>,
        setting: Setting,
    ) -> Self {
        GameManager {
            rooms: Arc::new(DashMap::new()),
            pool,
            redis,
            client_registry,
            setting,
        }
    }

    pub async fn ensure_room(&self , room_id: &str , max_players: usize) -> Arc<RwLock<RoomState>>{
        if let Some(ev) = self.rooms.get(room_id) {
            ev.value().clone()
        }else{
            let s = Arc::new(RwLock::new(RoomState::new(room_id, max_players)));
            self.rooms.insert(room_id.to_string(), s.clone());
            s
        }
    }

    
}


