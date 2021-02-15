use lazy_static::lazy_static;
use std::sync::Mutex;
use std::net::{SocketAddr, TcpStream};
use uuid::Uuid;
use cfb8::Cfb8;
use aes::Aes128;

lazy_static!(
    pub static ref PLAYERS: Mutex<Vec<Player>> = Mutex::new(Vec::new());
);

pub struct Player {
    pub connection: PlayerConnection,
    pub uuid: Uuid,
    pub nickname: String,
    pub encryption: Cfb8<Aes128>
}

pub struct PlayerConnection {
    pub addr: SocketAddr,
    pub stream: TcpStream
}