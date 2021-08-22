use std::time::Duration;
use std::thread::JoinHandle;
use crate::game::player::{PlayerList, Player};
use std::sync::MutexGuard;
use crate::net::packet_listener::PacketListenerStruct;
use crate::game::game_chat;
use crate::game::packets::Packet;
use crate::net::newer_network_manager::{NetWriter, GameProtocol};
use std::sync::mpsc::Receiver;
use crate::game::chat::ChatComponent;
use crate::game::player_join;

pub fn start(players: PlayerList, net_writer: NetWriter, game_reader: Receiver<GameProtocol>) -> JoinHandle<()> {
    //Ticks
    std::thread::Builder::new().name("Amethyst - Server Thread".to_owned()).spawn(move || {
        let packet_listeners = [
            // PacketListenerStruct {packet_id: 0x00, listener: network_manager::keep_alive_listener},
            PacketListenerStruct {packet_id: 0x01, listener: game_chat::chat_listener}
        ];
        let mut keep_alive_ticks = 0u8;

        loop {
            //Locks for sync environment
            let mut sync_environment =  SyncEnvironment {
                players: players.lock().unwrap()
            };
            // network_manager::tick(&mut sync_environment, &packet_listeners, &mut keep_alive_ticks);

            for message in game_reader.try_iter() {
                match message {
                    //Disconnect order from the net thread, it may be because of socket errors/disconnect, keep alive not sent, etc.
                    GameProtocol::ForcedDisconnect {token} => {
                        let index = sync_environment.players.iter().position(|player| player.token.eq(&token));
                        match index {Some(t) => { sync_environment.players.remove(t); }, None => {}};
                    }
                    GameProtocol::Login {token, nickname, uuid} => {
                        //Check if another player with the same UUID is already on the server
                        let already_logged_in = sync_environment.players.iter().any(|player| player.uuid.eq(&uuid));
                        if already_logged_in {
                            net_writer.disconnect(token, ChatComponent::new_text("You're already logged in!".to_string()));
                            continue;
                        }

                        let mut player = Player {token, nickname, uuid};
                        player_join::handle_join(&mut player, &net_writer);
                        sync_environment.players.push(player);
                    }
                    GameProtocol::Packet {token, packet} => {

                    }
                }
            }

            drop(sync_environment);
            //You need to drop everything before this sleep
            std::thread::sleep(Duration::from_millis(50));
        }
    }).unwrap()
}

pub struct SyncEnvironment<'a> {
    pub players: MutexGuard<'a, Vec<Player>>
}