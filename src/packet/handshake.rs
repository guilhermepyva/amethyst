use crate::packet::{ReadPacket, Packet};
use crate::data_reader::DataReader;
use crate::net::network_manager::MinecraftClient;
use std::sync::Arc;

#[derive(Debug)]
pub struct PacketHandshake {
    client: Arc<MinecraftClient>,
    protocol_version: u32,
    server_address: String,
    server_port: u16,
    next_state: u8
}

impl ReadPacket for PacketHandshake {
    fn read<'a>(mut reader: DataReader, client: Arc<MinecraftClient>) -> Result<Packet, &'a str> {
        let err = Result::Err("packet byte order is wrong!");

        Ok(Packet::Handshake(PacketHandshake {
            client,
            protocol_version: match reader.read_varint() { Ok(t) => t, Err(_e) => return err },
            server_address: match reader.read_string() { Ok(t) => t, Err(_e) => return err },
            server_port: match reader.read_u16() { Ok(t) => t, Err(_e) => return err},
            next_state: match reader.read_u8() { Ok(t) => t, Err(_e) => return err },
        }))
    }
}