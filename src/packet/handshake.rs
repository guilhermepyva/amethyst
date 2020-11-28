use uuid::Uuid;
use crate::packet::{ReadPacket, Packet, PacketStruct};
use crate::packet::RawPacket;

#[derive(Debug)]
pub struct PacketHandshake {
    packet: PacketStruct,
    protocol_version: u32,
    server_address: String,
    server_port: u16,
    next_state: u8
}

impl ReadPacket for PacketHandshake {
    fn read(raw_packet: RawPacket, uuid: Option<Uuid>) -> Result<Packet, &'static str> {
        let mut reader = raw_packet.get_reader();
        let err = "packet byte order is wrong!";

        Ok(Packet::Handshake(PacketHandshake {
            packet: PacketStruct { id: raw_packet.id, uuid },
            protocol_version: reader.read_varint().expect(err),
            server_address: reader.read_string().expect(err),
            server_port: reader.read_u16().expect(err),
            next_state: reader.read_u8().expect(err),
        }))
    }
}