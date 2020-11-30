use crate::packet;
use crate::utils::arrays;
use std::net::{TcpListener, TcpStream, SocketAddr};
use lazy_static::lazy_static;
use uuid::Uuid;
use std::sync::{Mutex, MutexGuard};
use std::error::Error;
use std::time::Duration;
use io::Read;
use std::io;
use std::borrow::BorrowMut;
use std::thread::JoinHandle;

const BUFFER_SIZE: usize = 512;

struct MinecraftClient {
    uuid: Uuid,
    stream: TcpStream,
    addr: SocketAddr
}

impl MinecraftClient {
    fn read_stream(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.read(buf)
    }
}

struct ReceivedData {
    read: usize,
    data: Vec<u8>
}

lazy_static!(
    static ref CLIENTS: Mutex<Vec<MinecraftClient>> = Mutex::new(vec![]);
    static ref PACKETS_RECEIVED: Mutex<Vec<ReceivedData>> = Mutex::new(vec![]);
);

pub trait PacketListener {
    fn received(&self, packet: &packet::Packet);
}

pub fn start() -> JoinHandle<()> {
    let server = TcpListener::bind("0.0.0.0:25565").unwrap();
    println!("Aguardando conexões");

    std::thread::spawn(move || {
        loop {
            let client = match server.accept() {
                Ok(t) => t,
                Err(e) => {
                    println!("Error while accepting client: {}", e.to_string());
                    continue;
                }
            };

            client.0.set_nonblocking(true);

            println!("Client conectou: {}", client.1.ip());
            CLIENTS.lock().unwrap().push(MinecraftClient {uuid: Uuid::new_v4(), stream: client.0, addr: client.1});
        }
    });

    let sleep_time = Duration::from_millis(5);
    let mut buf: [u8; BUFFER_SIZE] = [0; BUFFER_SIZE];
    std::thread::spawn(move || {
        let mut list_to_insert: Vec<ReceivedData> = Vec::with_capacity(128);
        let mut client_to_remove: Option<Uuid> = None;
        loop {
            let mut clients_locked = CLIENTS.lock().unwrap();
            for client in clients_locked.iter_mut() {
                loop {
                    let read = match client.read_stream(&mut buf) {
                        Ok(t) => t,
                        Err(e) => {
                            match e.kind() {
                                io::ErrorKind::WouldBlock => {
                                    break;
                                }
                                _ => {
                                    println!("An error occurred while reading data in Minecraft Clients: {}", e.to_string());
                                    break;
                                }
                            }
                        }
                    };

                    //Connection closed
                    if read == 0 {
                        client_to_remove = Some(client.uuid.clone());
                        break;
                    }

                    list_to_insert.push(ReceivedData {read, data: arrays::extract_vector(&buf, 0, read)})
                }
            }

            if client_to_remove.is_some() {
                let client_to_remove_unwrapped = client_to_remove.unwrap();
                let index = clients_locked.iter().position(|x| x.uuid == client_to_remove_unwrapped).unwrap();
                let disconnected = clients_locked.remove(index);
                client_to_remove = None;
                println!("Client desconectou: {}", disconnected.addr.ip())
            }

            drop(clients_locked);

            if !list_to_insert.is_empty() {
                PACKETS_RECEIVED.lock().unwrap().append(&mut list_to_insert);
            }
            std::thread::sleep(sleep_time);
        }
    })
}