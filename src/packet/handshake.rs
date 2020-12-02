use crate::packet::{ReadPacket, Packet};
use crate::data_reader::DataReader;
use crate::net::network_manager::MinecraftClient;
use std::sync::Arc;

#[derive(Debug)]
pub struct PacketHandshake {
    pub protocol_version: u32,
    pub server_address: String,
    pub server_port: u16,
    pub next_state: u8
}

impl ReadPacket for PacketHandshake {
    fn read<'a>(mut reader: DataReader) -> Result<Packet, &'a str> {
        Ok(Packet::Handshake(PacketHandshake {
            protocol_version: reader.read_varint()?,
            server_address: reader.read_string()?,
            server_port: reader.read_u16()?,
            next_state: reader.read_u8()?,
        }))
    }
}