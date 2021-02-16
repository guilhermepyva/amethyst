use std::time::Duration;
use crate::net::network_manager;
use std::thread::JoinHandle;
use crate::player::PlayerList;

pub fn start(players: PlayerList) -> JoinHandle<()> {
    //Ticks
    let duration = Duration::from_millis(50);
    std::thread::Builder::new().name("Amethyst - Server Thread".to_owned()).spawn(move || {
        loop {
            let sync_environment =  SyncEnvironment {};
            network_manager::tick_read_packets(sync_environment);

            std::thread::sleep(duration);
        }
    }).unwrap()
}

pub struct SyncEnvironment {
}