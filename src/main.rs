use std::sync::{Arc, Mutex, Weak};

use game::player::PlayerList;

use crate::net::packet_listener::PacketListenerStruct;
use crate::game::position::Position;
use std::mem::{size_of_val, size_of};
use std::sync::mpsc::{channel, Sender};
use crate::net::network_manager::{GameProtocol, NetProtocol, NetWriter};
use crate::game::packets::Packet;
use crate::game::chat::ChatComponent;

mod data_reader;
mod net;
mod data_writer;
mod game;


fn main() {
    let players: PlayerList = Box::leak(Box::new(Mutex::new(Vec::new())));
    let (net_writer, game_reader) = channel::<GameProtocol>();
    let (game_writer, net_reader) = channel::<NetProtocol>();

    let writer = NetWriter {writer: game_writer};

    net::network_manager::start(net_writer, net_reader);

    // net::https::test();
    game::engine::start(players, writer, game_reader).join().expect("couldn't join thread in main thread");
}