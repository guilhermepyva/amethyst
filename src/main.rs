use std::sync::{Arc, Mutex};
use crate::player::PlayerList;
use crate::net::packet_listener::PacketListenerStruct;
use crate::net::network_manager::KeepAliveListener;

mod packets;
mod data_reader;
mod utils;
mod net;
mod data_writer;
mod game;
mod player;

fn main() {
    let players: PlayerList = Arc::new(Mutex::new(Vec::new()));
    net::network_manager::start(players.clone());

    game::engine::start(players.clone()).join().expect("couldn't join thread in main thread");
}