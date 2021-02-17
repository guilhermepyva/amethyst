use std::time::Duration;
use crate::net::network_manager;
use std::thread::JoinHandle;
use crate::player::{PlayerList, Player};
use std::sync::MutexGuard;

pub fn start(players: PlayerList) -> JoinHandle<()> {
    //Ticks
    let duration = Duration::from_millis(50);
    std::thread::Builder::new().name("Amethyst - Server Thread".to_owned()).spawn(move || {
        loop {
            let mut sync_environment =  SyncEnvironment {
                players: players.lock().unwrap()
            };
            network_manager::tick_read_packets(&mut sync_environment);

            std::thread::sleep(duration);
        }
    }).unwrap()
}

pub struct SyncEnvironment<'a> {
    pub players: MutexGuard<'a, Vec<Player>>
}