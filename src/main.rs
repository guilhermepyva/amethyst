use std::sync::{Arc, Mutex};

use game::player::PlayerList;

use crate::net::network_manager::KeepAliveListener;
use crate::net::packet_listener::PacketListenerStruct;
use crate::game::position::Position;
use std::mem::{size_of_val, size_of};

mod data_reader;
mod utils;
mod net;
mod data_writer;
mod game;

fn main() {
    let players: PlayerList = Arc::new(Mutex::new(Vec::new()));
    net::network_manager::start(players.clone());

    // net::https::test();
    game::engine::start(players.clone()).join().expect("couldn't join thread in main thread");
}