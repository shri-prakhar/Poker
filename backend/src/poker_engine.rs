use std::{cmp::Ordering, collections::HashMap};

use argon2::password_hash::rand_core::{OsRng, RngCore};
use rand::{SeedableRng, rngs::StdRng, seq::SliceRandom};
use serde::{Deserialize, Serialize};

#[derive(Debug , Clone, Copy , PartialEq, Eq , Hash , Serialize , Deserialize)]
pub struct Card{
  pub rank : u8, // we will rank according to 2-14 , where 14 for Ace 
  pub suit : Suit
}

#[derive(Debug , Clone, Copy , PartialEq, Eq , Hash , Serialize  , Deserialize)]
pub enum Suit {
   Clubs,
   Diamonds,
   Hearts,
   Spades 
}

impl Card{
  pub fn to_string(&self) -> String {
    let r = match self.rank {
        11 =>  "J".to_string(),
        12 =>  "Q".to_string(),
        13 =>  "K".to_string(),
        14 =>  "A".to_string(),
        v => v.to_string()
    };

    let s = match self.suit {
        Suit::Clubs => "C".to_string(),
        Suit::Diamonds => "D".to_string(),
        Suit::Hearts => "H".to_string(),
        Suit::Spades => "S".to_string()
    };

    format!("{}{}" , r , s)
  }
}

impl From<&str> for Card {
    fn from(_s: &str) -> Self{
      unimplemented!()
    } 
}

#[derive(Debug , Clone, PartialEq, Eq)]
pub struct HandRank{
   category: u8, // 8 = straight flush , 7 = four of a kind , 6 = full house , 5 = flush , 4 = straight , 3 = three of a kind , 2 = two pair , 1 = one pair , 0 = high card 
   tiebreakers: Vec<u8> // in decreasing order of the rank (14..2)
}

impl Ord  for HandRank  {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.category.cmp(&other.category) {
            std::cmp::Ordering::Equal => self.tiebreakers.cmp(&other.tiebreakers),
            ord => ord
        }
    }
}
impl PartialOrd for HandRank{
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
      Some(self.cmp(other))
  }
}

pub fn new_deck() -> Vec<Card>{
  let mut deck =  Vec::with_capacity(52);
  for &s in &[Suit::Clubs , Suit::Diamonds , Suit::Hearts , Suit::Spades] {
    for r in 2u8..=14u8{
      deck.push(Card{rank: r , suit : s});
    }
  }
  deck
}

pub fn shuffle_deck(deck: &mut [Card]){
  let mut seed = [0u8; 32];
  OsRng.fill_bytes(&mut seed); //generates a random cryptographic bytes based on Os/System entropy
  let mut rng = StdRng::from_seed(seed); // generates a random number 
  deck.shuffle(&mut rng); 
}

pub fn deal_hole_cards(deck: &mut Vec<Card> , seats: usize) -> Vec<Option<(Card ,Card)>>{
  let mut out = vec![None; seats];
  let mut index = 0;
  for i in 0..seats {
    if deck.len() < index + 2 { break; }
    out[i] = Some((deck[index] , deck[index + 1]));
    index += 2 
  }
  deck.drain(0..seats);
  out
}

pub fn deal_flop(deck: &mut Vec<Card>) -> Vec<Card>{
  let mut out = Vec::new();
  if deck.len() >= 3 {
    out.push(deck.remove(0)); //so it removes the element and returns it value at that index
    out.push(deck.remove(0));
    out.push(deck.remove(0));
  }
  out
}
pub fn deal_turn(deck: &mut Vec<Card>) -> Option<Card>{
  if deck.is_empty(){ return None; }
  Some(deck.remove(0))
}

pub fn deal_river(deck: &mut Vec<Card>) -> Option<Card>{
  if deck.is_empty(){
    return None;
  }
  Some(deck.remove(0))
}

pub fn evaluate_five(cards: &[Card; 5]) -> HandRank {
  let mut ranks = cards.iter().map(|c| c.rank).collect::<Vec<u8>>();
  ranks.sort_by(|a , b| b.cmp(a)); //descending

  let mut freq = HashMap::<u8 , usize>::new();
  for &r in &ranks {
    *freq.entry(r).or_insert(0) += 1;
  }

  let mut suits = HashMap::<Suit , usize>::new();
  for &c in cards.iter() {
    *suits.entry(c.suit).or_insert(0) += 1;
  }

  let is_flush = suits.values().any(|&v| v == 5);
  let mut unique_ranks = ranks.clone();
  unique_ranks.dedup(); //dedup makes matching entries as one 
  let mut is_straight = false;
  let mut top_straight = 0u8;
  let mut ranks_for_straight = unique_ranks.clone();
  if unique_ranks.contains(&14){
    ranks_for_straight.push(1);
  }

  ranks_for_straight.sort_by(|a , b| b.cmp(a));

  for i in 0..=(ranks_for_straight.len().saturating_sub(5)){
    let ok = true;
    for j in 0..4{
      if ranks_for_straight[i+j] != ranks_for_straight[i + j + 1] + 1{
        is_straight = false;
        break;  
      } 
    }
    if ok{
      is_straight = true;
      top_straight = ranks_for_straight[i];
      break;
    }
  }

  let mut counts = freq.iter().map(|(&r , &c)| (c as u8 , r)).collect::<Vec<_>>();
  counts.sort_by(|a,b| {
    match a.0.cmp(&b.0) {
        Ordering::Equal => b.1.cmp(&a.1),
        ord => ord
    }
  }
  );

  //straight flush
  if is_flush && is_straight {
    return HandRank { category: 8, tiebreakers: vec![top_straight]};
  }

  //four of a kind 
  if counts[0].0 == 4{
    let four_rank = counts[0].1;
    let kicker = *ranks.iter().find(|&&r| r != four_rank).unwrap();
    return HandRank { category: 7, tiebreakers: vec![four_rank , kicker] };
  }
  
  //full house 3 + 2
  if counts[0].0 == 3 && counts.len() >= 2 && counts[1].0 == 2{
    let triple = counts[0].1;
    let pair = counts[1].1;
    return HandRank { category: 6, tiebreakers: vec![triple , pair] };
  }

  //flush
  if is_flush {
    return HandRank { category: 5, tiebreakers: ranks.clone()};
  }

  //straight
  if is_straight{
    return HandRank { category: 4, tiebreakers: vec![top_straight]};
  }

  //Three of a kind 
  if counts[0].0 == 3{
    let triple = counts[0].1;
    let kickers = ranks.iter().filter(|&&r| r != triple).cloned().collect::<Vec<_>>();
    return HandRank { category: 3, tiebreakers: [vec![triple] , kickers].concat() };
  }

  //two pair 
  if counts[0].0 == 2 && counts.len() >=2 && counts[1].0 == 2 {
    let high_pair = counts[0].1;
    let low_pair = counts[1].1;
    let kicker = *ranks.iter().find(|&&r| r != low_pair && r != high_pair).unwrap(); 
    HandRank { category: 2, tiebreakers: vec![high_pair , low_pair , kicker] };
  }

  //one pair
  if counts[0].0 == 2 {
    let pair_rank = counts[0].1;
    let kickers = ranks.iter().filter(|&&r| r != pair_rank).cloned().collect::<Vec<_>>();
    return HandRank { category: 1, tiebreakers: [vec![pair_rank] , kickers].concat() };
  } 

  //high card
  HandRank { category: 0, tiebreakers: ranks.clone() }
}


pub fn evaluate_best_of_seven(cards: [Card; 7]) -> HandRank {
  let n = cards.len();
  assert!(n>=5 && n<=7 , "cards length must be less than 7 and greater than 5");
  let mut best = None;
  let mut indexes = Vec::new();
  for a in 0..n{
    for b in a..n{
      for c in b..n{
        for d in c..n{
          for e in  d..n{
            indexes.clear();
            indexes.push(a);indexes.push(b);indexes.push(c);indexes.push(d);indexes.push(e);
            let hand = [cards[indexes[0]],cards[indexes[1]],cards[indexes[2]],cards[indexes[3]],cards[indexes[4]]];
            let rank = evaluate_five(&hand);
            match best {
              None => best = Some(rank),
              Some(ref cur) => if rank > *cur { best = Some(rank)} // this comparison is based on ord trait implemented on HandRank it first compares against category then tiebreakers
            }
          }
        }
      }
    }
  }
  best.expect("at least one five card hand")
}

