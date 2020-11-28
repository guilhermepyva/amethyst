use uuid::Uuid;
use crate::datareader::DataReader;

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
    fn read(raw_packet: RawPacket, uuid: Option<Uuid>) -> Result<Packet, &'static str>;
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

pub fn get_packet(raw_packet: RawPacket, uuid: Option<Uuid>) -> Result<Packet, &'static str> {
    match raw_packet.id {
        0x00 => login_start::PacketLoginStart::read(raw_packet, uuid),
        _ => Err("Packet id not programmed")
    }
}