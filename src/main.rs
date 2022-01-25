use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read, Write};
use std::sync::{Arc, Mutex, Weak};

use game::player::PlayerList;

use crate::game::chat::ChatComponent;
use crate::game::packets::Packet;
use crate::net::network_manager::{GameProtocol, NetProtocol, NetWriter};
use crate::net::packet_listener::PacketListenerStruct;
use game::world::coords::Position;
use std::mem::{size_of, size_of_val};
use std::sync::mpsc::{channel, Sender};
use flate2::read::GzDecoder;
use fxhash::{FxBuildHasher, FxHashMap};
use crate::game::nbt::NBTTag;

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

    let file = BufReader::new(File::open("level.dat").unwrap());
    let mut decoder = GzDecoder::new(file);

    let nbt = NBTTag::read(&mut decoder, true);
    println!("name {:?}", nbt);

    // net::https::test();
    game::engine::start(players, writer, game_reader);
}
