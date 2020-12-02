use crate::packet::{ReadPacket, Packet};
use crate::data_reader::DataReader;
use crate::net::network_manager::MinecraftClient;
use std::sync::Arc;

#[derive(Debug)]
pub struct PacketLoginStart {
    name: String
}

impl ReadPacket for PacketLoginStart {
    fn read<'a>(mut reader: DataReader) -> Result<Packet, &'a str> {
        Ok(Packet::LoginStart(PacketLoginStart { name: reader.read_string()?, }))
    }
}