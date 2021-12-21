use crate::data_writer::DataWriter;
use crate::game::chat::ChatComponent;
use crate::game::packets::Packet;
use aes::cipher::StreamCipher;
use aes::Aes128;
use cfb8::Cfb8;
use mio::Token;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

pub struct Player {
    pub token: Token,
    pub uuid: Uuid,
    pub nickname: String,
}

pub type PlayerList = &'static Mutex<Vec<Player>>;
