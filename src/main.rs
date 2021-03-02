use std::sync::{Arc, Mutex};

use game::player::PlayerList;

use crate::net::network_manager::KeepAliveListener;
use crate::net::packet_listener::PacketListenerStruct;
use crate::game::position::Position;

mod data_reader;
mod utils;
mod net;
mod data_writer;
mod game;

fn main() {
    let players: PlayerList = Arc::new(Mutex::new(Vec::new()));
    net::network_manager::start(players.clone());

    println!("{}", Position {x: 50, y: 70, z: 50}.encode());
    println!("{}", Position {x: 780, y: 450, z: 8000}.encode());
    game::engine::start(players.clone()).join().expect("couldn't join thread in main thread");
}