use uuid::Uuid;
use crate::data_reader::DataReader;
use crate::net::network_manager::MinecraftClient;
use std::sync::Arc;

pub mod handshake;
pub mod login_start;
pub mod encryption_request;

#[derive(Debug)]
pub enum Packet{
    Handshake(handshake::PacketHandshake),
    LoginStart(login_start::PacketLoginStart),
    EncryptionRequest(encryption_request::PacketEncryptionRequest)
}

#[derive(Debug)]
pub struct PacketStruct {
    pub id: u8,
    pub uuid: Option<Uuid>
}

pub trait ReadPacket {
    fn read<'a>(reader: DataReader, client: Arc<MinecraftClient>) -> Result<Packet, &'a str>;
}

pub trait WritePacket {
    fn write(&self) -> Vec<u8>;
}

#[derive(Debug)]
pub struct RawPacket {
    pub id: u8,
    pub data: Vec<u8>
}

impl RawPacket {
    pub fn get_reader(&self) -> DataReader {
        return DataReader::new(&self.data);
    }
}

pub fn get_packet(id: u32, mut reader: DataReader, client: Arc<MinecraftClient>) -> Result<Packet, &'static str> {
    match id {
        0x00 => login_start::PacketLoginStart::read(reader, client),
        _ => Err("Packet id not programmed")
    }
}