use std::net::{TcpListener, Shutdown};
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::io::Read;
use crate::{packet, utils};
use crate::packet::{Packet, ReadPacket};

const BUFFER_SIZE: usize = 512;

pub trait PacketListener {
    fn received(&self, packet: &packet::Packet);
}

lazy_static!(
    static ref LISTENERS: Mutex<Vec<Box<dyn PacketListener + Send>>> = Mutex::new(vec![]);
);

pub fn register_listener(listener: impl PacketListener + Send + 'static) {
    LISTENERS.lock().unwrap().push(Box::new(listener));
}

pub fn start() {
    let server = TcpListener::bind("127.0.0.1:25565").unwrap();

    loop {
        println!("Esperando conexões");
        let mut client = server.accept().unwrap().0;
        let mut handshake = false;

        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        loop {
            if client.read(&mut buf).unwrap() == 0 {
                break;
            }

            let rawpacket = packet::RawPacket {
                id: buf[1], data: utils::arrays::array_copy(&buf, 2, (buf[0] + 1) as usize)
            };

            let packet = if !handshake && rawpacket.id == 0 {
                handshake = true;
                packet::packet_handshake::PacketHandshake::read(rawpacket, None)
            } else {
                packet::get_packet(rawpacket, None)
            };

            let packet = match packet {
                Err(e) => {
                    println!("Error while reading packet: {}", e);
                    continue;
                }
                Ok(t) => t
            };

            for listener in LISTENERS.lock().unwrap().iter() {
                listener.received(&packet);
            }
        }

        client.shutdown(Shutdown::Both);
    }
}