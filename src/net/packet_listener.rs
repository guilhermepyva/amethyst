use crate::game::packets::Packet;
use crate::game::player::Player;
use crate::game::engine::SyncEnvironment;

pub struct PacketListenerStruct {
    pub packet_id: i32,
    pub listener: Box<dyn PacketListener>
}

pub trait PacketListener {
    fn listen(&self, packet: &Packet, player_index: usize, environment: &mut SyncEnvironment);
}