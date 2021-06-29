use std::time::Duration;
use crate::net::network_manager;
use std::thread::JoinHandle;
use crate::game::player::{PlayerList, Player};
use std::sync::MutexGuard;
use crate::net::packet_listener::PacketListenerStruct;
use crate::net::network_manager::KeepAliveListener;
use crate::game::game_chat::ChatListener;

pub fn start(players: PlayerList) -> JoinHandle<()> {
    //Ticks
    let duration = Duration::from_millis(50);
    std::thread::Builder::new().name("Amethyst - Server Thread".to_owned()).spawn(move || {
        let packet_listeners = vec![
            PacketListenerStruct {packet_id: 0x00, listener: Box::new(KeepAliveListener {})},
            PacketListenerStruct {packet_id: 0x01, listener: Box::new(ChatListener {})}
        ];
        let mut keep_alive_ticks = 0u8;

        loop {
            let mut sync_environment =  SyncEnvironment {
                players: players.lock().unwrap()
            };
            network_manager::tick(&mut sync_environment, &packet_listeners, &mut keep_alive_ticks);

            drop(sync_environment);
            //You need to drop everything before this sleep
            std::thread::sleep(duration);
        }
    }).unwrap()
}

pub struct SyncEnvironment<'a> {
    pub players: MutexGuard<'a, Vec<Player>>
}