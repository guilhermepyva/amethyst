use uuid::Uuid;
use crate::packet::{ReadPacket, Packet, PacketStruct};
use crate::data_reader::DataReader;
use crate::net::network_manager::MinecraftClient;
use std::sync::Arc;

#[derive(Debug)]
pub struct PacketLoginStart {
    client: Arc<MinecraftClient>,
    name: String
}

impl ReadPacket for PacketLoginStart {
    fn read<'a>(mut reader: DataReader, client: Arc<MinecraftClient>) -> Result<Packet, &'a str> {
        let err = "packet byte order is wrong!";

        Ok(Packet::LoginStart(PacketLoginStart {
            client,
            name: reader.read_string().expect(err),
        }))
    }
}