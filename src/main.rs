use crate::packet::Packet;
use crate::net::server;

mod packet;
mod datareader;
mod utils;
mod net;

fn main() {
    server::register_listener(PackageDisplay {});
    server::start()
}

struct PackageDisplay {}

impl server::PacketListener for PackageDisplay {
    fn received(&self, packet: &Packet) {
        println!("{:?}", packet);
    }
}