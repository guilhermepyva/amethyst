use crate::net::network_manager::MinecraftClient;
use std::sync::Arc;
use crate::packet::{ReadPacket, Packet};
use crate::data_reader::DataReader;

#[derive(Debug)]
pub struct PingPacket {
    pub ping: i64
}
impl ReadPacket for PingPacket {
    fn read<'a>(mut reader: DataReader) -> Result<Packet, &'a str> {
        Ok(Packet::Ping(PingPacket{ ping: reader.read_i64()? }))
    }
}