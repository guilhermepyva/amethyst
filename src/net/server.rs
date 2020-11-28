use std::net::{TcpListener, Shutdown, TcpStream};
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::io::{Read, Write};
use crate::{packet, utils};
use crate::packet::ReadPacket;
use crate::utils::arrays::array_copy;
use crate::net::login::LoginPacketListener;

const BUFFER_SIZE: usize = 512;

pub trait PacketListener {
    fn received(&self, packet: &packet::Packet);
}

lazy_static!(
    //É uma ideia fazer uma lista não mutex que copia essa aqui pra não ser modificada depois do servidor ter sido aberto
    static ref LISTENERS: Mutex<Vec<Box<dyn PacketListener + Send>>> = Mutex::new(vec![]);
    static ref CLIENT: Mutex<Option<TcpStream>> = Mutex::new(None);
);

static mut SEND_BUFFER: [u8; 512] = [0; 512];

pub fn send_bytes(bytes: Vec<u8>) {
    unsafe {
        SEND_BUFFER = [0; 512];
        array_copy(bytes, &mut SEND_BUFFER);
        CLIENT.lock().unwrap().as_mut().unwrap().write(&SEND_BUFFER);
    }
}

pub fn register_listener(listener: impl PacketListener + Send + 'static) {
    LISTENERS.lock().unwrap().push(Box::new(listener));
}

pub fn start() {
    register_listener(LoginPacketListener{});

    let server = TcpListener::bind("127.0.0.1:25565").unwrap();

    loop {
        println!("Esperando conexões");
        *CLIENT.lock().unwrap() = Some(server.accept().unwrap().0);
        let mut handshake = false;

        let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];

        loop {
            if CLIENT.lock().unwrap().as_mut().unwrap().read(&mut buf).unwrap() == 0 {
                break;
            }

            let rawpacket = packet::RawPacket {
                id: buf[1],
                data: utils::arrays::extract_vector(&buf, 2, (buf[0] + 1) as usize)
            };

            let packet = if !handshake && rawpacket.id == 0 {
                handshake = true;
                packet::handshake::PacketHandshake::read(rawpacket, None)
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

        CLIENT.lock().unwrap().as_ref().unwrap().shutdown(Shutdown::Both);
    }
}