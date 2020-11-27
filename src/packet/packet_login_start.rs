use uuid::Uuid;
use crate::packet::{ReadPacket, Packet, PacketStruct};
use crate::packet::RawPacket;

#[derive(Debug)]
pub struct PacketLoginStart {
    packet: PacketStruct,
    name: String
}

impl ReadPacket for PacketLoginStart {
    fn read(raw_packet: RawPacket, uuid: Option<Uuid>) -> Result<Packet, &'static str> {
        let mut reader = raw_packet.get_reader();
        let err = "packet byte order is wrong!";

        Ok(Packet::LoginStart(PacketLoginStart {
            packet: PacketStruct { id: raw_packet.id, uuid },
            name: reader.read_string().expect(err),
        }))
    }
}