use crate::game::player::Player;
use crate::game::packets::{Packet, PlayerInfoPlayer, PlayerInfoAction, WorldBorderAction, Slot};
use crate::game::position::Position;
use crate::game::nbt::{NBTTag, CompoundElement};
use crate::game::chat::ChatComponent;
use crate::data_writer::DataWriter;
use std::time::Duration;
use crate::game::ray_tracing::{PosValue, ray_casting, print_matrix};

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

pub fn handle_join(player: &mut Player) {
    player.connection.send_packet(&Packet::KeepAlive {id: 0});
    player.connection.send_packet(&Packet::JoinGame {
        entity_id: 0,
        gamemode: 1,
        dimension: 0,
        difficulty: 0,
        max_players: 255,
        level_type: "teste".to_string(),
        reduced_debug_info: false
    });
    player.connection.send_packet(&Packet::SpawnPosition {location: Position {
        x: 0,
        y: 50,
        z: 0
    }});
    player.connection.send_packet(&Packet::HeldItemChange {slot: 0});
    player.connection.send_packet(&Packet::PlayerInfo {action_id: 0, players: vec!(PlayerInfoPlayer {
        uuid: player.uuid.clone(),
        action: PlayerInfoAction::AddPlayer {
            name: player.nickname.clone(),
            properties: vec!(),
            gamemode: 0,
            ping: 0,
            display_name: Option::from(ChatComponent::new_text(player.nickname.clone()))
        }
    })});
    println!("{}", player.nickname);
    player.connection.send_packet(&Packet::PlayerPositionAndLook {
        x: 0.0,
        y: 50.0,
        z: 0.0,
        yaw: 0.0,
        pitch: 0.0,
        flags: 0
    });
    player.connection.send_packet(&Packet::WorldBorder {action: WorldBorderAction::SetSize {radius: 100f64}});
    player.connection.send_packet(&Packet::TimeUpdate {world_age: 0, time_of_day: 12000});

    let bitmask = 0b0000000000001000 as u16;
    let mut writer = DataWriter::new();
    let mut blocks = [[[0u16; 16]; 16]; 16];
    let stone = (1 << 4) | 0;
    let torch = (50 << 4) | 5;
    let dirt = (3 << 4) | 0;

    for z in 0..16 {
        for x in 0..16 {
            blocks[0][z][x] = stone;
        }
    }
    blocks[1][8][8] = torch;
    blocks[1][8][7] = stone;

    let mut matrix = [[0i8; 16]; 16];
    let light_torch = PosValue::new(8, 8, 13);
    PosValue::new(7, 8, -1).set(&mut matrix);
    light_torch.set(&mut matrix);

    ray_casting(&mut matrix, &light_torch);

    let mut lightning = [[[5u8; 16]; 16]; 16];

    for z in 0..16 {
        for x in 0..16 {
            if matrix[x][z] < 0 {
                continue;
            }
            lightning[1][z][x] = matrix[x][z] as u8;
        }
    }

    let mut block_light = [0u8; 2048];
    let mut i = 0;
    for y in 0..16 {
        for z in 0..16 {
            for x in (0..16).step_by(2) {
                block_light[i] = ((lightning[y][z][x + 1] << 4) + lightning[y][z][x]);
                i += 1;
            }
        }
    }
    let mut skylight = [0u8; 2048];

    player.connection.send_packet(&Packet::ChunkData {
        bitmask,
        ground_up_continuous: true,
        x: 0,
        y: 0,
        data: write_chunk_light(&blocks, &block_light, &skylight)
    });
    //
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
    //
    // player.connection.send_packet(&Packet::ChunkData {
    //     x: 0,
    //     y: 0,
    //     ground_up_continuous: false,
    //     bitmask,
    //     data: write_chunk(&blocks, 12, 13)
    // });


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

pub fn write_chunk_light(blocks: &[[[u16; 16]; 16]; 16], block_light: &[u8; 2048], sky_light: &[u8; 2048]) -> Vec<u8> {
    let mut writer = DataWriter::new();
    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                writer.write_u16_le(blocks[y][z][x]);
            }
        }
    }
    writer.write_data(&block_light.to_vec());
    writer.write_data(&sky_light.to_vec());
    for x in 0..256 {
        writer.write_u8(1);
    }

    writer.data
}

pub fn write_chunk(blocks: &[[[u16; 16]; 16]; 16], block_light: u8, sky_light: u8) -> Vec<u8> {
    let mut writer = DataWriter::new();
    for y in 0..16 {
        for z in 0..16 {
            for x in 0..16 {
                writer.write_u16_le(blocks[y][z][x]);
            }
        }
    }
    for x in 0..2048 {
        writer.write_u8((block_light << 4) | block_light);
    }
    for x in 0..2048 {
        writer.write_u8((sky_light << 4) | sky_light);
    }
    for x in 0..256 {
        writer.write_u8(1);
    }

    writer.data
}