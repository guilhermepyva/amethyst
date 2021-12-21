use crate::data_writer::DataWriter;
use crate::game::chat::ChatComponent;
use crate::game::engine::SyncEnvironment;
use crate::game::nbt::{CompoundElement, NBTTag};
use crate::game::packets::{Packet, PlayerInfoAction, PlayerInfoPlayer, Slot, WorldBorderAction};
use crate::game::player::Player;
use crate::game::world::angle::Angle;
use crate::game::world::block::Material;
use crate::game::world::chunk::{ChunkPos, ChunkSection};
use crate::game::world::coords::{Point, Position};
use crate::net::network_manager::NetWriter;
use std::mem::size_of_val;
use std::time::{Duration, Instant};

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
19 - window items
 */

pub fn handle_join(player: &mut Player, net_writer: &NetWriter, environment: &mut SyncEnvironment) {
    println!(
        "Player {} ({}) joined the server",
        player.nickname, player.uuid
    );
    let token = player.token;
    net_writer.send_packet(token, Packet::KeepAlive { id: 0 });
    net_writer.send_packet(
        token,
        Packet::JoinGame {
            entity_id: 0,
            gamemode: 1,
            dimension: 0,
            difficulty: 0,
            max_players: 255,
            level_type: environment.world.level_type.to_str().to_string(),
            reduced_debug_info: false,
        },
    );
    net_writer.send_packet(
        token,
        Packet::SpawnPosition {
            location: Position { x: 0, y: 50, z: 0 },
        },
    );
    net_writer.send_packet(token, Packet::HeldItemChange { slot: 0 });
    net_writer.send_packet(
        token,
        Packet::PlayerInfo {
            action_id: 0,
            players: vec![PlayerInfoPlayer {
                uuid: player.uuid.clone(),
                action: PlayerInfoAction::AddPlayer {
                    name: player.nickname.clone(),
                    properties: vec![],
                    gamemode: 0,
                    ping: 0,
                    display_name: Option::from(ChatComponent::new_text(player.nickname.clone())),
                },
            }],
        },
    );
    net_writer.send_packet(
        token,
        Packet::PlayerPositionAndLook {
            x: 0.0,
            y: 51.0,
            z: 0.0,
            yaw: 0.0,
            pitch: 0.0,
            flags: 0,
        },
    );
    net_writer.send_packet(
        token,
        Packet::WorldBorder {
            action: WorldBorderAction::SetSize { radius: 100f64 },
        },
    );
    net_writer.send_packet(
        token,
        Packet::TimeUpdate {
            world_age: 0,
            time_of_day: 12000,
        },
    );

    let now = Instant::now();
    for x in environment.world.chunks.iter() {
        net_writer.send_packet(token, x.write_chunk_data());
    }
    println!("{}", now.elapsed().as_nanos());

    net_writer.send_packet(token, Packet::KeepAlive { id: 4 });

    // net_writer.send_packet(token, Packet::SpawnObject {
    //     id: 69,
    //     object: 60,
    //     point: Point {x: 0f64, y: 60f64, z: 0f64},
    //     angle: Angle {pitch: 20, yaw: 20},
    //     data: 0,
    //     vel_x: None,
    //     vel_y: None,
    //     vel_z: None
    // })

    // let mut id = 256;
    // for y in 0..16 {
    //     for z in 0..16 {
    //         for x in 0..16 {
    //             blocks[y][z][x] = (id << 4) | 0;
    //             id += 1;
    //         }
    //     }
    // }
    //
    // std::thread::sleep(Duration::from_secs(1));
    //
    // let data = vec![0; (4096 * 2) + (4096 + 256)];

    // player.connection.send_packet(&Packet::WindowItems {window_id: 0, slots: vec!(
    //     Slot {
    //         item_id: 1,
    //         item_count: Some(1),
    //         item_damage: Some(69),
    //         nbt: Some(NBTTag::Compound {
    //             compound: vec!(CompoundElement {
    //                 name: "display".to_owned(),
    //                 tag: NBTTag::Compound {
    //                     compound: vec!(
    //                         CompoundElement {name: "Name".to_owned(), tag: NBTTag::String {string: "Pyrocah".to_owned()}},
    //                     )
    //                 }
    //             })
    //         })
    //     }
    // )});
}
