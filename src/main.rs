use std::sync::{Arc, Mutex, Weak};

use game::player::PlayerList;

use crate::game::chat::ChatComponent;
use crate::game::packets::Packet;
use crate::net::network_manager::{GameProtocol, NetProtocol, NetWriter};
use crate::net::packet_listener::PacketListenerStruct;
use game::world::coords::Position;
use std::mem::{size_of, size_of_val};
use std::sync::mpsc::{channel, Sender};

mod data_reader;
mod data_writer;
mod game;
mod net;

fn main() {
    let players: PlayerList = Box::leak(Box::new(Mutex::new(Vec::new())));
    let (net_writer, game_reader) = channel::<GameProtocol>();
    let (game_writer, net_reader) = channel::<NetProtocol>();

    let writer = NetWriter {
        writer: game_writer,
    };

    net::network_manager::start(net_writer, net_reader);

    let opt = Some(4);

    // net::https::test();
    game::engine::start(players, writer, game_reader);
}
