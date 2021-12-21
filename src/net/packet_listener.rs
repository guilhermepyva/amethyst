use crate::game::{engine::SyncEnvironment, packets::Packet, player::Player};

pub type PacketListener = fn(&Packet, usize, &mut SyncEnvironment);

pub struct PacketListenerStruct {
    pub packet_id: i32,
    pub listener: PacketListener,
}
