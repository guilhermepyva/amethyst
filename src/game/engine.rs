use crate::game::chat::ChatComponent;
use crate::game::game_chat;
use crate::game::packets::Packet;
use crate::game::player::{Player, PlayerList};
use crate::game::player_join;
use crate::game::world::chunk::ChunkPos;
use crate::game::world::generator::generate;
use crate::game::world::world::{LevelType, World};
use crate::net::network_manager::{GameProtocol, NetWriter};
use crate::net::packet_listener::PacketListenerStruct;
use std::ops::DerefMut;
use std::sync::mpsc::Receiver;
use std::sync::MutexGuard;
use std::thread::JoinHandle;
use std::time::Duration;

pub fn start(
    players: PlayerList,
    net_writer: NetWriter,
    game_reader: Receiver<GameProtocol>,
) -> ! {
    let packet_listeners = [
        // PacketListenerStruct {packet_id: 0x00, listener: network_manager::keep_alive_listener},
        PacketListenerStruct {
            packet_id: 0x01,
            listener: game_chat::chat_listener,
        },
    ];
    let mut keep_alive_ticks = 0u8;

    let mut world = World::new("Mundo".to_string(), 0, LevelType::Default);
    generate(&mut world);

    //Ticks
    loop {
        //Locks for sync environment
        {
            let mut players = players.lock().unwrap();

            let mut sync_environment = SyncEnvironment {
                players: players.deref_mut(),
                world: &mut world,
            };
            // network_manager::tick(&mut sync_environment, &packet_listeners, &mut keep_alive_ticks);

            for message in game_reader.try_iter() {
                match message {
                    //Disconnect order from the net thread, it may be because of socket errors/disconnect, keep alive not sent, etc.
                    GameProtocol::ForcedDisconnect { token, reason } => {
                        let index = sync_environment
                            .players
                            .iter()
                            .position(|player| player.token.eq(&token));
                        match index {
                            Some(t) => {
                                let player = sync_environment.players.remove(t);
                                println!("Player {} disconnected", player.nickname)
                            }
                            None => {}
                        };
                    }
                    GameProtocol::Login {
                        token,
                        nickname,
                        uuid,
                    } => {
                        //Check if another player with the same UUID is already on the server
                        let already_logged_in = sync_environment
                            .players
                            .iter()
                            .any(|player| player.uuid.eq(&uuid));
                        if already_logged_in {
                            net_writer.disconnect(
                                token,
                                ChatComponent::new_text("You're already logged in!".to_string()),
                            );
                            continue;
                        }

                        let mut player = Player {
                            token,
                            nickname,
                            uuid,
                        };
                        player_join::handle_join(&mut player, &net_writer, &mut sync_environment);
                        sync_environment.players.push(player);
                    }
                    GameProtocol::Packet { token, packet } => {}
                }
            }
        }
        //You need to drop everything before this sleep
        std::thread::sleep(Duration::from_millis(50));
    }
}

pub struct SyncEnvironment<'a> {
    pub players: &'a mut Vec<Player>,
    pub world: &'a mut World,
}
