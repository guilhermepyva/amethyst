use crate::player::Player;
use crate::packets::Packet;

pub fn handle_join(player: &mut Player) {
    player.connection.send_packet(Packet::JoinGame {
        entity_id: 5,
        gamemode: 0,
        dimension: 0,
        difficulty: 0,
        max_players: 255,
        level_type: "teste".to_string(),
        reduced_debug_info: false
    })
}