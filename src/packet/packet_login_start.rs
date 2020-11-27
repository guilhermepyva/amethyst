use uuid::Uuid;
use crate::packet::{ReadPacket, Packet};
use crate::packet::rawpacket::RawPacket;

#[derive(Debug)]
pub struct PacketLoginStart {
    packet: Packet,
    name: String
}

impl ReadPacket<PacketLoginStart> for PacketLoginStart {
    fn read(raw_packet: RawPacket, uuid: Option<Uuid>) -> Result<PacketLoginStart, &'static str> {
        let mut reader = raw_packet.get_reader();
        let err = "packet byte order is wrong!";

        Ok(PacketLoginStart {
            packet: Packet { id: raw_packet.id, uuid },
            name: reader.read_string().expect(err),
        })
    }
}