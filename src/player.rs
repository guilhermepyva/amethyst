use std::sync::{Mutex, Arc};
use std::net::{SocketAddr, TcpStream};
use uuid::Uuid;
use cfb8::Cfb8;
use aes::Aes128;

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

pub type PlayerList = Arc<Mutex<Vec<Player>>>;