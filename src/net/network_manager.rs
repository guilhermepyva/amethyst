use crate::packet;
use crate::utils::arrays;
use std::net::{TcpListener, TcpStream, SocketAddr};
use lazy_static::lazy_static;
use uuid::Uuid;
use std::sync::{Mutex, MutexGuard, Arc};
use std::error::Error;
use std::time::Duration;
use io::Read;
use std::io;
use std::borrow::BorrowMut;
use std::thread::JoinHandle;
use crate::datareader::DataReader;
use crate::packet::ReadPacket;
use crate::net::login::LoginPacketListener;

const BUFFER_SIZE: usize = 8192;

#[derive(Debug)]
pub struct MinecraftClient {
    uuid: Uuid,
    handshake: Mutex<bool>,
    addr: SocketAddr
}

struct Connection {
    properties: Arc<MinecraftClient>,
    stream: TcpStream
}


impl Connection {
    fn read_stream(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

#[derive(Clone)]
struct RawPacket {
    client: Arc<MinecraftClient>,
    id: u32,
    data: Vec<u8>
}

pub trait PacketListener {
    fn received(&self, packet: &packet::Packet);
}

lazy_static!(
    static ref CLIENTS: Mutex<Vec<Connection>> = Mutex::new(vec![]);
    static ref PACKETS_RECEIVED: Mutex<Vec<RawPacket >> = Mutex::new(vec![]);
    static ref LISTENERS: Mutex<Vec<Box<dyn PacketListener + Send>>> = Mutex::new(vec![]);
);

pub fn register_listener(listener: impl PacketListener + Send + 'static) {
    LISTENERS.lock().unwrap().push(Box::new(listener));
}

pub fn start() {
    let server = match TcpListener::bind("0.0.0.0:25565") {
        Ok(t) => t,
        Err(e) => {
            println!("Error while binding server: {}", e);
            return;
        }
    };
    println!("Aguardando conexões");

    std::thread::Builder::new().name("Amethyst - Client Handler Thread".to_owned()).spawn(move || {
        loop {
            let client = match server.accept() {
                Ok(t) => t,
                Err(e) => {
                    println!("Error while accepting client: {}", e.to_string());
                    continue;
                }
            };

            match client.0.set_nonblocking(true) {
                Err(e) => {
                    println!("Couldn't set nonblocking to {}: {}", client.1.ip(), e);
                    continue;
                }
                _ => {}
            };

            println!("Client conectou: {}", client.1.ip());
            CLIENTS.lock().unwrap().push(Connection {properties: Arc::new(MinecraftClient {uuid: Uuid::new_v4(), addr: client.1, handshake: Mutex::new(false)}), stream: client.0});
        }
    });

    let sleep_time = Duration::from_millis(5);
    let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    std::thread::Builder::new().name("Amethyst - Packet Handler Thread".to_owned()).spawn(move || {
        let mut list_to_insert: Vec<RawPacket> = Vec::with_capacity(128);
        let mut client_to_remove: Option<Uuid> = None;
        loop {
            let mut clients_locked = CLIENTS.lock().unwrap();
            for client in clients_locked.iter_mut() {
                let read = match client.read_stream(&mut buf) {
                    Ok(t) => t,
                    Err(e) => {
                        match e.kind() {
                            io::ErrorKind::WouldBlock => {
                                continue;
                            }
                            _ => {
                                println!("An error occurred while reading data in Minecraft Clients: {}", e.to_string());
                                continue;
                            }
                        }
                    }
                };

                //Connection closed
                if read == 0 {
                    client_to_remove = Some(client.properties.clone().uuid);
                    continue;
                }

                let data_vec = arrays::extract_vector(&buf, 0, read);
                let mut reader = DataReader::new(&data_vec);
                list_to_insert.append(&mut match read_packets(&mut reader, read, &client.properties) {
                    Ok(t) => t,
                    Err(e) => {
                        //TODO Disconnect por não ter conseguido ler packets
                        continue;
                    }
                });
            }

            if client_to_remove.is_some() {
                let client_to_remove_unwrapped = client_to_remove.unwrap();
                let index = clients_locked.iter().position(|x| x.properties.uuid == client_to_remove_unwrapped).unwrap();
                let disconnected = clients_locked.remove(index);
                client_to_remove = None;
                println!("Client desconectou: {}", disconnected.properties.addr.ip())
            }

            drop(clients_locked);

            if !list_to_insert.is_empty() {
                PACKETS_RECEIVED.lock().unwrap().append(&mut list_to_insert);
            }
            std::thread::sleep(sleep_time);
        }
    });

    register_listener(LoginPacketListener {});
}

pub fn tick_read_packets() {
    let mut packets_received_locked = PACKETS_RECEIVED.lock().unwrap();
    let packets = (*packets_received_locked).clone();
    packets_received_locked.clear();
    drop(packets_received_locked);

    for packet_data in packets {
        //TODO Disconnect se tiver errado a sequencia
        let mut reader = DataReader::new(&packet_data.data);
        let mut handshake = packet_data.client.handshake.lock().unwrap();

        let packet = if !*handshake {
            if packet_data.id == 0 {
                *handshake = true;
                match packet::handshake::PacketHandshake::read(reader, packet_data.client.clone()) {
                    Ok(t) => t,
                    Err(e) => {
                        println!("{}", e);
                        //TODO Disconnect por handshake invalido
                        continue;
                    }
                }
            } else {
                //TODO Disconnect
                continue;
            }
        } else {
            match packet::get_packet(packet_data.id, reader, packet_data.client.clone()) {
                Ok(t) => t,
                Err(e) => {
                    println!("{}", e);
                    //TODO Disconnect por erro ao ler o packet de ID {}
                    continue;
                }
            }
        };

        drop(handshake);

        for listener in LISTENERS.lock().unwrap().iter() {
            listener.received(&packet);
        }
    }
}

fn read_packets<'a>(reader: &mut DataReader, read: usize, client: &Arc<MinecraftClient>) -> Result<Vec<RawPacket>, &'a str> {
    let mut vec = vec![];

    let mut jump_bytes: usize = 0;
    while reader.cursor != read {
        let length = reader.read_varint()?;

         vec.push(RawPacket {
             client: client.clone(),
             id: reader.read_varint()?,
             data: reader.read_data_fixed((length + 1 - (reader.cursor - jump_bytes) as u32) as usize)?
         });
        jump_bytes += reader.cursor;
    }

    Ok(vec)
}