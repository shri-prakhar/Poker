use std::sync::{Arc, RwLock};

use anyhow::Ok;
use chrono::Utc;
use dashmap::DashMap;
use database::models::{add_player, create_hand, finish_hand, hand_players, insert_action, insert_player, remove_players};
use redis::aio::ConnectionManager;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, any};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{config::Setting, poker_engine::{Card, HandRank, evaluate_best_of_seven, new_deck, shuffle_deck}, ws_server::ClientInfo};

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
    pub hole_cards: Vec<Option<(Card, Card)>>, //per seat
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
            for slot in r.seats.iter_mut(){
                if let Some(ps) = slot{
                    if ps.user_id == user_id{
                        let _ = remove_players(&self.pool, room_id, ps.seat as i16).await;
                        //TODO:: Implement emit event
                        *slot = None;
                        break;
                    }
                }
            }
        }
        Ok(())
    }
    
    pub async fn start_hand(&self , room_id: Uuid) -> anyhow::Result<Uuid>{
        let entry = self.rooms.get(&room_id).ok_or_else(|| anyhow::anyhow!("room not found")).unwrap();
        let mut r = entry.value().write().map_err(|e| anyhow::anyhow!("room already occupied , {}" , e))?;
        let mut deck = new_deck();
        shuffle_deck(&mut deck);
        let mut hole_cards = vec![None; r.seats.len()];
        let mut deck_cards = 0usize;
        for (i , slot) in r.seats.iter().enumerate(){
            if slot.is_some(){
                let card1 = deck.remove(0);
                let card2 = deck.remove(0);
                hole_cards[i] = Some((card1 , card2));
            }
        }
        let hand_id = Uuid::new_v4();
        let started_at = Utc::now();
        let players_in_hand = r.seats.iter().map(|slot| slot.is_some()).collect::<Vec<_>>();
        let hand = HandState{
            id: hand_id,
            started_at: started_at.timestamp_millis(),
            pot : 0,
            board: Vec::new(),
            hole_cards: hole_cards.clone(),
            current_turn: Some(Self::first_to_act_index(&r)),
            round: "pre-flop".to_string(),
            players_in_hand
        };
        let _ = create_hand(&self.pool, Some(room_id), Some(started_at)).await;
        for (i , slot) in r.seats.iter().enumerate(){
            if let Some(ps) = slot{
                if let Some((card1 , card2)) = &hole_cards[i]{
                    let hole_j = serde_json::json!([card1.to_string() , card2.to_string()]);
                    let _ = insert_player(&self.pool, Some(hand_id), (i + 1) as i16, Some(ps.user_id), Some(hole_j), Some(ps.chips), Some(ps.chips)).await;
                }
            }
        }
        r.active_hand = Some(hand);
        drop(r);
        //TODO::spawn_timer
        //TODO::Implement emit events
        Ok(hand_id)
    }

    fn first_to_act_index(r: &RoomState) -> usize{
        if let Some(d) = r.dealer_index {
            let n = r.seats.len();
            for i in 1..=n {
                let index = (d + i) % n;
                if r.seats[index].is_some(){ return index; }
            }
            0   
        }else{
            for (i , s) in r.seats.iter().enumerate(){
                if s.is_some() { return i; }
            }
            0
        }
    }

        pub async fn handle_action(&self , user_id: Uuid , room_id: Uuid , action: serde_json::Value) -> anyhow::Result<()>{
            let entry = self.rooms.get(&room_id).ok_or_else(|| anyhow::anyhow!("room not found"))?;
            let mut r = entry .value().write().map_err(|e| anyhow::anyhow!("Can't Write {}" , e ))?;
            let seat_index = r.seats.iter().position(|s| s.as_ref().map(|p| p.user_id == user_id).unwrap_or(false)).ok_or_else(|| anyhow::anyhow!("player not found"))?;
            {
                let hs = r.active_hand.as_ref().ok_or_else(|| anyhow::anyhow!("no active hand"))?;
                //let seat = r.seats[seat_index].as_mut();
                if Some(seat_index) != hs.current_turn {
                    return Err(anyhow::anyhow!("it's not player's turn"));
                }
            }
            let action_type = action.get("type").and_then(|v| v.as_str()).unwrap_or("unknown").to_string();
            let amount = action.get("amount").and_then(|v| v.as_i64()).unwrap_or(0);
            //let hs = r.active_hand.as_mut().ok_or_else(|| anyhow::anyhow!("no active hand"))?;
            match action_type.as_str(){
                "fold" => {
                    let hs = r.active_hand.as_mut().ok_or_else(|| anyhow::anyhow!("no active hand"))?;
                    hs.players_in_hand[seat_index] = false;
                    let _  = insert_action(&self.pool, Some(hs.id), Some(user_id), Some(action_type), Some(amount)).await;
                    //TODO:: Implement emit event   
                },
                "bet" | "raise" | "call" => {
                    
                    if let Some(ps) = r.seats[seat_index].as_mut(){
                        ps.chips -= amount;
                    }
                    let hs = r.active_hand.as_mut().ok_or_else(|| anyhow::anyhow!("no active hand"))?; 
                    hs.pot += amount;
                    let _ = insert_action(&self.pool, Some(hs.id), Some(user_id), Some(action_type), Some(amount)).await;
                    //TODO:: Implement emit event
                },
                "check" => {
                    let hs = r.active_hand.as_ref().ok_or_else(|| anyhow::anyhow!("no active hand"))?;
                    let _ = insert_action(&self.pool, Some(hs.id), Some(user_id), Some(action_type), Some(amount)).await;
                    //TODO:: Implement emit event
                },
                "allin" => {
                    if let Some(ps) = r.seats[seat_index].as_mut(){
                        let amt = ps.chips;
                        ps.chips = 0;
                        let hs = r.active_hand.as_mut().ok_or_else(|| anyhow::anyhow!("no active hand"))?;
                        hs.pot += amt;
                        let _ = insert_action(&self.pool, Some(hs.id), Some(user_id), Some(action_type), Some(amount)).await;
                        //TODO:: Implement emit event
                    }   
                },
                other => {
                    return Err(anyhow::anyhow!("Unsupported Action {}" , other));
                }
            }
            let next_turn = {
                let hs = r.active_hand.as_mut().ok_or_else(|| anyhow::anyhow!("no active hand"))?;
                let next = Self::next_live_player(&hs.players_in_hand, seat_index);
                hs.current_turn = next;
            };
            
            //TODO: spawn timer
            if let Some(entry2) = self.rooms.get(&room_id){
                let r2 = entry2.value().read().map_err(|e| anyhow::anyhow!("room not available {}" , e))?;
                let alive = r2.active_hand.as_ref().unwrap().players_in_hand.iter().filter(|&&p| p).count();
                if alive <= 1{
                    drop(r2);
                    //TODO:: finish hand
                }
            }
            Ok(())
        }

        pub async fn finish_hand(&self , room_id: Uuid) -> anyhow::Result<()>{
            let entry = self.rooms.get(&room_id).ok_or_else(|| anyhow::anyhow!("room not found"))?;
            let mut r = entry.value().write().map_err(|e| anyhow::anyhow!("not availble for write {}" , e))?;
            let hs = r.active_hand.take().ok_or_else(|| anyhow::anyhow!("no active hand"))?;
            let mut best_rank: Option<(usize , HandRank)> = None;
            for (i, alive) in hs.players_in_hand.iter().enumerate(){
                if !*alive { continue; }
                if let Some((c1 , c2)) = &hs.hole_cards[i]{
                    let mut cards = hs.board.clone();
                    cards.push(*c1);
                    cards.push(*c2);
                    let rank = evaluate_best_of_seven(&cards);
                    match &best_rank{
                        None => best_rank = Some((i , rank)),
                        Some((_ , rprev)) => {
                            if rank > *rprev { best_rank = Some((i , rank))}
                        }
                    }
                }
            }

            if let Some((winner_index , rank)) = best_rank{
                if let Some(Some(ps)) = r.seats.get_mut(winner_index){
                    ps.chips += hs.pot;
                    let winner_id = ps.user_id;
                    let time = Utc::now();
                    let _ = finish_hand(&self.pool, hs.id, Some(time), hs.pot, None, Some(winner_id), Some(serde_json::json!({"pot": hs.pot}))).await;
                    //TODO: emit event
                }else{
                    let time = Utc::now();
                    let _ = finish_hand(&self.pool, hs.id, Some(time), hs.pot, None, None, Some(serde_json::json!({"pot": hs.pot}))).await;
                    //TODO: emit event 
                }
            }
            Ok(())
        }

    fn next_live_player(player_in_hand: &Vec<bool> , from: usize) -> Option<usize>{
        let n = player_in_hand.len();
        if n==0 { return None;}
        for i in 1..=n{
            let index = (from + i) % n ;
            if player_in_hand[index] { return Some(index);}
        }
        None
    }
}


