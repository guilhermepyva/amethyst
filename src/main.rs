use std::sync::{Arc, Mutex, Weak};

use game::player::PlayerList;

use crate::net::packet_listener::PacketListenerStruct;
use crate::game::position::Position;
use std::mem::{size_of_val, size_of};

mod data_reader;
mod net;
mod data_writer;
mod game;

fn main() {
    let players: PlayerList = Box::leak(Box::new(Mutex::new(Vec::new())));
    net::network_manager::start(players);

    // net::https::test();
    game::engine::start(players).join().expect("couldn't join thread in main thread");
}