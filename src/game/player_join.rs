use crate::player::Player;
use crate::packets::{Packet, PlayerInfoPlayer, PlayerInfoAction};
use crate::game::position::Position;
use std::thread::Thread;
use std::time::Duration;

/*
36 - join game
23 - n - plugin message
13 - n - server difficulty
66 - spawn position
48 - n - player abilities
63 - held item change
6 - n - statistics
14 - n - chat message
50 - player info (add player)
50 - player info (update latency)
52 - player position and look
61 - world border
78 - time update
19 - player position and rotation
21 - player movement
21 - player movement
 */

pub fn handle_join(player: &mut Player) {
    player.connection.send_packet(Packet::KeepAlive {id: 69});
    std::thread::sleep(Duration::from_millis(200));
    player.connection.send_packet(Packet::JoinGame {
        entity_id: 5,
        gamemode: 0,
        dimension: 0,
        difficulty: 0,
        max_players: 255,
        level_type: "teste".to_string(),
        reduced_debug_info: false
    });
    player.connection.send_packet(Packet::SpawnPosition {location: Position {
        x: 0,
        y: 50,
        z: 0
    }});
    player.connection.send_packet(Packet::HeldItemChange {slot: 0});
    player.connection.send_packet(Packet::PlayerInfo {action_id: 0, players: vec!(PlayerInfoPlayer {
        uuid: player.uuid.clone(),
        action: PlayerInfoAction::AddPlayer {
            name: player.nickname.clone(),
            properties: vec!(),
            gamemode: 0,
            ping: 0,
            display_name: None
        }
    })});
    player.connection.send_packet(Packet::PlayerPositionAndLook {
        x: 0.0,
        y: 50.0,
        z: 0.0,
        yaw: 0.0,
        pitch: 0.0,
        flags: 0
    });
}