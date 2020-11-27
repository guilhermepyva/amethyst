use std::net::{TcpListener, Shutdown};
use std::io::Read;
use packet::ReadPacket;
use crate::packet::packet_handshake::PacketHandshake;

mod packet;
mod datareader;
mod utils;

const BUFFER_SIZE: usize = 512;

fn main() {
    let server = TcpListener::bind("127.0.0.1:25565").unwrap();

    loop {
        println!("Esperando conexões");
        let mut client = server.accept().unwrap().0;

        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        let mut login = false;
        for _ in 0..2 {
            if client.read(&mut buf).unwrap() == 0 {
                break;
            }

            let rawpacket = packet::rawpacket::RawPacket {
                id: buf[1], data: utils::arrays::array_copy(&buf, 2, (buf[0] + 1) as usize).unwrap()
            };

            if !login {
                println!("{:?}", packet::packet_handshake::PacketHandshake::read(rawpacket, None).unwrap());
            } else {
                println!("{:?}", packet::packet_login_start::PacketLoginStart::read(rawpacket, None).unwrap());
            }

            login = true;
        }

        client.shutdown(Shutdown::Both);
    }
}