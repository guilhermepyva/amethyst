use uuid::Uuid;
use crate::packet::rawpacket::RawPacket;

pub mod rawpacket;
pub mod packet_handshake;
pub mod packet_login_start;
pub mod handler;

#[derive(Debug)]
struct Packet {
    id: u8,
    uuid: Option<Uuid>
}

pub trait ReadPacket<T> {
    fn read(raw_packet: RawPacket, uuid: Option<Uuid>) -> Result<T, &'static str>;
}