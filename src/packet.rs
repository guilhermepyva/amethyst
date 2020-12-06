use crate::data_reader::DataReader;
use crate::net::network_manager::ConnectionState;

pub mod handshake;
pub mod login_start;
pub mod encryption_request;
pub mod disconnect_login;
pub mod disconnect_play;
pub mod status_request;
pub mod status_response;
pub mod ping;
pub mod pong;
pub mod encryption_response;
pub mod login_success;

#[derive(Debug)]
pub enum Packet{
    Handshake(handshake::PacketHandshake),
    StatusRequest,
    Ping(ping::PingPacket),
    LoginStart(login_start::PacketLoginStart),
    EncryptionResponse(encryption_response::PacketEncryptionResponse)
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
                0x00 => Ok(Packet::StatusRequest),
                0x01 => ping::PingPacket::read(reader),
                _ => Err("Packet id not programmed")
            }
        }
        _ => Err("Packet state not programmed")
    }
}