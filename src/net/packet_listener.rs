use crate::game::{player::Player, packets::Packet, engine::SyncEnvironment};

pub type PacketListener = fn(&Packet, usize, &mut SyncEnvironment);

pub struct PacketListenerStruct {
    pub packet_id: i32,
    pub listener: PacketListener
}