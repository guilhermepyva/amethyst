use crate::game::player::PlayerList;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Poll, Token, Interest};
use std::time::Duration;
use std::net::{SocketAddr, Shutdown};
use mio::event::Source;
use std::io::{Read, ErrorKind};
use crate::data_reader::DataReader;

//Server address
const ADDR: &str = "127.0.0.1:25565";

//Token for epoll identification
const SERVER_TOKEN: Token = Token(0);

const BUFFER_SIZE: usize = 4096;

pub fn start(players: PlayerList) {
    //Open server
    let mut server = TcpListener::bind(ADDR.parse().unwrap()).expect("An error occured while binding the server");

    //Initialize epoll
    let mut poll = Poll::new().expect("An error occured while initializing the epoll");
    let mut events = Events::with_capacity(1024);

    //Register the server
    poll.registry().register(&mut server, SERVER_TOKEN, Interest::READABLE);

    //Client list
    let mut clients = Vec::new();
    let mut token_counter = 1usize;

    println!("Waiting for connections on {}", ADDR);

    std::thread::Builder::new().name("IO Network Thread".to_string()).spawn(move || {
        let mut buffer = [0u8; BUFFER_SIZE];

        loop {
            //Poll events
            poll.poll(&mut events, None);

            if events.is_empty() {continue;}

            for event in events.iter() {
                let token = event.token();

                //Check server event
                if token == SERVER_TOKEN {
                    //Loop to accept all clients and finish server reading
                    loop {
                        match server.accept() {
                            //Got a client
                            Ok(mut client) => {
                                //Check if client is already logging
                                if clients.iter().any(|x: &(usize, TcpStream, SocketAddr) | client.1.ip().eq(&x.2.ip())) {
                                    //TODO Disconnect client
                                    println!("ih");
                                }

                                //Register client in poll and clients vector
                                poll.registry().register(&mut client.0, Token(token_counter), Interest::READABLE);
                                clients.push((token_counter, client.0, client.1));
                                token_counter += 1;

                                println!("Client connected: {}", client.1.ip())
                            }

                            //Check if it reached the end
                            Err(ref err) if err.kind() == ErrorKind::WouldBlock => break,

                            //Check for another error
                            Err(e) => println!("An error occured while accepting a client: {}", e)
                        }
                    }
                } else {
                    //Check for clients token
                    let client = clients.iter_mut().find(|client| client.0 == token.0);

                    match client {
                        Some(client) => {
                            let mut disconnect = false;

                            //Check for connection states first, this may not trigger in some platforms,
                            //thats why we still keep track on EOF and read 0 while reading the stream
                            if event.is_read_closed() {disconnect = true}
                            else if event.is_error() {
                                disconnect = true;
                                println!("An error occured in client {} socket, told by the epoll", client.2.ip())
                            }

                            //Read values in buffer and copy to vector
                            //first_read to check if, in the first read, the result is 0, meaning that the client has disconnected
                            let mut first_read = true;
                            let mut vec = Vec::with_capacity(2048);

                            if !disconnect {
                                loop {
                                    let read = match client.1.read(&mut buffer) {
                                        Ok(t) => t,

                                        //Read end
                                        Err(ref e) if e.kind() == ErrorKind::WouldBlock => break,

                                        //EOF
                                        Err(ref e) if e.kind() == ErrorKind::UnexpectedEof => {
                                            disconnect = true;
                                            break;
                                        },

                                        //Check another error
                                        Err(e) => {
                                            println!("An error occured while reading a client's stream: {}", e);
                                            disconnect = true;
                                            break;
                                        }
                                    };

                                    if read == 0 && first_read {
                                        disconnect = true;
                                        break;
                                    }

                                    first_read = false;

                                    vec.extend_from_slice(&buffer[0..read]);
                                    if read != 4096 {
                                        break;
                                    }
                                }
                            }

                            if disconnect {
                                //TODO Disconnect
                                poll.registry().deregister(&mut client.1);
                                break;
                            }

                            let raw_packets = match read_packets(&vec) {
                                Some(t) => t,
                                None => { println!("Error while decoding client {} packets", client.2.ip()); continue; }
                            };
                        }
                        None => {}
                    }
                }
            }
        }
    });
}

pub struct RawPacket<'a> {
    pub id: i32,
    pub data: &'a [u8]
}

fn read_varint<'a>(slice: &[u8], index: &mut usize) -> Option<i32> {
    let mut result: i32 = 0;
    let mut read: u8;
    for i in 0..=5 {
        read = *slice.get(i)?;
        result += (((read as i8) & 0b01111111) as i32) << (7 * i);
        *index += 1;

        if (read & 0b10000000) == 0 {
            return Some(result)
        }
    }

    return None;
}

fn read_packets(data: &Vec<u8>) -> Option<Vec<RawPacket>> {
    let mut raw_packets = Vec::new();
    let mut index = 0usize;
    while index < data.len() {
        //Check if it has no space for packet length reading
        if index >= data.len() {return None};
        let mut length = read_varint(&data[index..], &mut index)? as usize;
        let mut id_length = 0usize;

        //Check if it has no space for id length reading
        if index >= data.len() {return None};
        let id = read_varint(&data[index..], &mut id_length)?;
        index += id_length as usize;
        length -= id_length;

        //Check if it has no space for reading the rest of the packet
        if index >= data.len() || length >= data.len() || index >= length {return None};
        raw_packets.push(RawPacket {id, data: &data[index..length]});
        index += length;
    }

    Some(raw_packets)
}