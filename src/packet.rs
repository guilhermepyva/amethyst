use uuid::Uuid;
use crate::datareader::DataReader;

pub mod packet_handshake;
pub mod packet_login_start;

#[derive(Debug)]
pub enum Packet{
    Handshake(packet_handshake::PacketHandshake),
    LoginStart(packet_login_start::PacketLoginStart)
}

#[derive(Debug)]
struct PacketStruct {
    id: u8,
    uuid: Option<Uuid>
}

pub trait ReadPacket {
    fn read(raw_packet: RawPacket, uuid: Option<Uuid>) -> Result<Packet, &'static str>;
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
        0x00 => packet_login_start::PacketLoginStart::read(raw_packet, uuid),
        _ => Err("Packet id not programmed")
    }
}