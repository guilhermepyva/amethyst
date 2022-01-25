use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Cursor, Read, Seek, SeekFrom, Write};
use std::sync::{Arc, Mutex, Weak};

use game::player::PlayerList;

use crate::game::chat::ChatComponent;
use crate::game::packets::Packet;
use crate::net::network_manager::{GameProtocol, NetProtocol, NetWriter};
use crate::net::packet_listener::PacketListenerStruct;
use game::world::coords::Position;
use std::mem::{size_of, size_of_val};
use std::sync::mpsc::{channel, Sender};
use byteorder::{BigEndian, ReadBytesExt};
use flate2::read::{GzDecoder, ZlibDecoder};
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

    let mut file = File::open("r.0.0.mca").unwrap();

    let mut location_table = [0u8; 4096];
    file.read(&mut location_table);
    location_table.reverse();

    let location_table = bytemuck::cast::<_, [i32; 1024]>(location_table);

    let chunk = 784;

    let offset = (location_table[chunk] >> 8) * 4096;
    let size_file = (location_table[chunk] & 0xFF) * 4096;

    file.seek(SeekFrom::Start(offset as u64));

    let size = file.read_i32::<BigEndian>().unwrap();
    let compression_scheme = file.read_i8().unwrap();
    println!("off {} size {}", offset, size_file);
    println!("{}", size);
    println!("{}", compression_scheme);

    let mut decoder = libflate::zlib::Decoder::new(file).unwrap();

    // let mut vec = Vec::new();
    // loop {
    //     let mut bytes = [0u8; 1024];
    //     let size = match decoder.read(&mut bytes) {
    //         Ok(t) => t,
    //         Err(e) => {
    //             println!("{:?}", e);
    //             break;
    //         }
    //     };
    //
    //     if size == 0 {
    //         break;
    //     }
    //
    //     vec.extend_from_slice(&bytes);
    // }
    //
    // let mut cursor = Cursor::new(vec);

    let nbt = NBTTag::read(&mut decoder, true, None);
    println!("{:?}", nbt);

    drop(decoder);

    // std::fs::remove_file("eita.mca");
    // let mut fileunc = File::create("eita.mca").unwrap();
    // loop {
    //     let mut bytes = [0u8; 1024];
    //     let size = match decoder.read(&mut bytes) {
    //         Ok(t) => t,
    //         Err(e) => {
    //             println!("{:?}", e);
    //             break;
    //         }
    //     };
    //
    //     if size == 0 {
    //         break;
    //     }
    //
    //     fileunc.write(&bytes[0..size]);
    //     fileunc.flush();
    // }
    // drop(fileunc);

    // file.read(&mut [0u8; 4]);
    //
    // let mut decoder = GzDecoder::new(file);
    //
    // let nbt = NBTTag::read(&mut decoder, true);
    // println!("{:?}", nbt);

    // net::https::test();
    game::engine::start(players, writer, game_reader);
}
