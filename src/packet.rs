use uuid::Uuid;
use crate::data_reader::DataReader;
use crate::net::network_manager::{MinecraftClient, ConnectionState};
use std::sync::Arc;

pub mod handshake;
pub mod login_start;
pub mod encryption_request;
pub mod disconnect_login;
pub mod disconnect_play;
mod status_request;
pub mod status_response;
mod ping;
pub mod pong;

#[derive(Debug)]
pub enum Packet{
    Handshake(handshake::PacketHandshake),
    StatusRequest(status_request::PacketStatusRequest),
    LoginStart(login_start::PacketLoginStart),
    EncryptionRequest(encryption_request::PacketEncryptionRequest),
    DisconnectLogin(disconnect_login::PacketDisconnectLogin),
    DisconnectPlay(disconnect_play::PacketDisconnectPlay),
    Ping(ping::PingPacket),
    Pong(pong::PongPacket)
}

pub trait ReadPacket {
    fn read<'a>(reader: DataReader) -> Result<Packet, &'a str>;
}

pub trait WritePacket {
    fn write(&self) -> Vec<u8>;
}

pub fn get_packet(id: u32, reader: DataReader, state: ConnectionState) -> Result<Packet, &'static str> {
    match state {
        ConnectionState::Login => {
            match id {
                0x00 => login_start::PacketLoginStart::read(reader),
                _ => Err("Packet id not programmed")
            }
        }
        ConnectionState::Status => {
            match id {
                0x00 => Ok(Packet::StatusRequest(status_request::PacketStatusRequest{})),
                0x01 => ping::PingPacket::read(reader),
                _ => Err("Packet id not programmed")
            }
        }
        _ => Err("Packet state not programmed")
    }
}