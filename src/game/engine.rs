use std::time::{Duration, Instant};
use crate::net::network_manager;
use std::thread::JoinHandle;

pub fn start() -> JoinHandle<()> {
    //Ticks
    let duration = Duration::from_millis(50);
    std::thread::Builder::new().name("Amethyst - Server Thread".to_owned()).spawn(move || {
        loop {
            network_manager::tick_read_packets();

            std::thread::sleep(duration);
        }
    }).unwrap()
}