use std::sync::{Mutex, Arc};
use std::net::{SocketAddr, TcpStream};
use uuid::Uuid;
use cfb8::Cfb8;
use aes::Aes128;
use crate::game::chat::ChatComponent;
use crate::packets::Packet;
use aes::cipher::StreamCipher;
use crate::data_writer::DataWriter;
use std::io::Write;

pub struct Player {
    pub connection: PlayerConnection,
    pub uuid: Uuid,
    pub nickname: String,
    pub join_game: bool
}

pub struct PlayerConnection {
    pub addr: SocketAddr,
    pub stream: TcpStream,
    pub encryption: Cfb8<Aes128>,
    pub shutdown: bool,
    pub disconnect: Option<ChatComponent>
}

impl PlayerConnection {
    pub fn disconnect(&mut self, reason: ChatComponent) {
        self.disconnect = Some(reason);
        self.shutdown = true;
    }

    pub fn shutdown(&mut self) {
        self.shutdown = true;
    }

    pub fn send_packet(&mut self, packet: Packet) {
        let mut packet_binary = packet.serialize().unwrap();
        packet_binary.splice(0..0, DataWriter::get_varint(packet_binary.len() as u32));
        self.encryption.encrypt(&mut packet_binary);
        self.stream.write(&packet_binary);
    }
}

pub type PlayerList = Arc<Mutex<Vec<Player>>>;