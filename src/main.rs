use crate::packet::Packet;
use crate::net::server;
use crate::data_writer::DataWriter;
use crate::data_reader::DataReader;
use std::net::TcpListener;
use std::io::Read;
use std::time::Duration;

mod packet;
mod data_reader;
mod utils;
mod net;
mod data_writer;
mod game;

fn main() {
    net::network_manager::start();
    game::engine::start().join();
}

struct PackageDisplay {}

impl server::PacketListener for PackageDisplay {
    fn received(&self, packet: &Packet) {
        println!("{:?}", packet);
    }
}