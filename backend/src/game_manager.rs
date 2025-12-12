use std::sync::{Arc, RwLock};

use anyhow::Ok;
use dashmap::DashMap;
use database::models::{add_player, remove_players};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{config::Setting, poker_engine::Card, ws_server::ClientInfo};

#[derive(Debug , Clone , Serialize , Deserialize)]
pub struct OutgoingEvent{
  pub event_type:String,
  pub room_id : Uuid,
  pub payload : serde_json::Value,
  pub emitted_at : i64
}
#[derive(Debug, Clone)]
pub struct PlayerSlot {
    pub user_id: Uuid,
    pub seat: usize,
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
    pub room_id: Uuid,
    pub max_players: usize,
    pub seats: Vec<Option<PlayerSlot>>,
    pub dealer_index: Option<usize>,
    pub active_hand: Option<HandState>,
    pub turn_task: Option<CancellationToken>,
}
impl RoomState {
    pub fn new(room_id: Uuid, max_players: usize) -> Self {
        RoomState {
            room_id: room_id,
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
    pub rooms: Arc<DashMap<Uuid, Arc<RwLock<RoomState>>>>,
    pub pool: PgPool,
    pub redis: ConnectionManager,
    pub client_registry: Arc<DashMap<Uuid, Vec<ClientInfo>>>,
    pub setting: Setting,
}

impl GameManager {
    pub async fn new(
        pool: PgPool,
        redis: ConnectionManager,
        client_registry: Arc<DashMap<Uuid, Vec<ClientInfo>>>,
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

    pub async fn ensure_room(&self , room_id: Uuid , max_players: usize) -> Arc<RwLock<RoomState>>{
        if let Some(ev) = self.rooms.get(&room_id) {
            ev.value().clone()
        }else{
            let s = Arc::new(RwLock::new(RoomState::new(room_id, max_players)));
            self.rooms.insert(room_id, s.clone());
            s
        }
    }

    pub async fn join_room(&self , user_id: Uuid ,  room_id: Uuid , requested_seat: Option<u8>) -> anyhow::Result<u8>{
        let room = self.ensure_room(room_id, 6).await;
        let mut r = room.write().map_err(|e| anyhow::anyhow!("not available {}",e))?;

        if let Some(req) = requested_seat{
            let index = (req - 1) as usize;
            if index < r.seats.len() && r.seats[index].is_none(){
                r.seats[index] = Some(PlayerSlot { user_id , seat: index , chips: 1000, connected: true });
                let _ = add_player(&self.pool, room_id , req as i16 , user_id, 1000, false).await;
                //TODO :: implement emit events
                return Ok(req);
            }
        }
            
            for (i , s) in r.seats.iter_mut().enumerate(){
                if s.is_none(){
                    let seat_num = (i + 1) as u8;
                    *s = Some(PlayerSlot { user_id, seat: seat_num as usize, chips: 1000, connected: true });
                    let _ = add_player(&self.pool, room_id, seat_num as i16, user_id, 1000, false).await;
                    //TODO:: Implement emit event 
                    return Ok(seat_num);
                }
            }
            Err(anyhow::anyhow!("room full"))
    }

    pub async fn leave_room(&self ,user_id: Uuid , room_id: Uuid) -> anyhow::Result<()>{
        if let Some(entry) = self.rooms.get(&room_id){
            let mut r = entry.value().write().map_err(|e| anyhow::anyhow!("not available {}" , e))?;
            for slot in &mut r.seats{
                if let Some(ps) = slot{
                    if ps.user_id == user_id{
                        *slot = None;
                        let _ = remove_players(&self.pool, room_id, ps.seat as i16).await;
                        //TODO:: Implement emit event
                        break;
                    }
                }
            }
        }
        Ok(())
    }
}


