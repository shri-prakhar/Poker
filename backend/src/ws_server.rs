use futures::channel::mpsc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum Outgoing {
    Text(String),
    Binary(Vec<u8>),
}

#[derive(Clone)]
pub struct ClientInfo {
    pub user_id: Uuid,
    pub tx: mpsc::Sender<Outgoing>,
}
