use std::sync::{Mutex, Arc};
use std::net::{SocketAddr, TcpStream};
use uuid::Uuid;
use cfb8::Cfb8;
use aes::Aes128;
use crate::game::chat::ChatComponent;
use crate::game::packets::Packet;
use aes::cipher::StreamCipher;
use crate::data_writer::DataWriter;
use std::io::Write;
use mio::Token;

pub struct Player {
    pub token: Token,
    pub uuid: Uuid,
    pub nickname: String
}

pub type PlayerList = &'static Mutex<Vec<Player>>;