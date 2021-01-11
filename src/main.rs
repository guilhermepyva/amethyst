mod packets;
mod data_reader;
mod utils;
mod net;
mod data_writer;
mod game;

fn main() {
    net::network_manager::start();
    game::engine::start().join().expect("couldn't join thread in main thread");
}