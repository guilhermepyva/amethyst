use crate::packet::Packet;
use crate::net::server;
use crate::datawriter::DataWriter;
use crate::datareader::DataReader;
use std::net::TcpListener;
use std::io::Read;
use std::time::Duration;

mod packet;
mod datareader;
mod utils;
mod net;
mod datawriter;
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