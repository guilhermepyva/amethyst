use crate::game::player::{PlayerList, PlayerConnection};
use crate::net::login_handler;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Poll, Token, Interest};
use std::time::Duration;
use std::net::{SocketAddr, Shutdown};
use mio::event::Source;
use std::io::{Read, ErrorKind, Write};
use crate::data_reader::DataReader;
use cfb8::Cfb8;
use aes::Aes128;
use uuid::Uuid;
use std::collections::HashMap;
use crate::game::packets::Packet;
use crate::data_writer::DataWriter;

//Server address
const ADDR: &str = "127.0.0.1:25565";

//Token for epoll identification
const SERVER_TOKEN: Token = Token(0);

const BUFFER_SIZE: usize = 4096;

pub struct Connection {
    pub token: Token,
    pub stream: TcpStream,
    pub addr: SocketAddr,
    pub identifier: String
}

pub struct PlayerLoginClient {
    pub connection: Connection,
    pub state: ConnectionState,
    pub nickname: Option<String>,
    pub verify_token: Option<[u8; 4]>,
    pub encode: Option<Cfb8<Aes128>>,
    pub decode: Option<Cfb8<Aes128>>,
    pub profile_uuid: Option<Uuid>
}

impl PlayerLoginClient {
    pub fn write(&mut self, packet: Packet) {
        let mut data = match packet.serialize() {Some(t) => t, None => return};
        data.splice(0..0, DataWriter::get_varint(data.len() as u32));
        self.connection.stream.write(&data);
    }
}

pub struct PlayerClient {
    connection: Connection
}

#[derive(Debug, Copy, Clone)]
pub enum ConnectionState {
    Handshaking,
    Status,
    Login,
    Play
}

pub fn start(players: PlayerList) {
    //Open server
    let mut server = TcpListener::bind(ADDR.parse().unwrap()).expect("An error occured while binding the server");

    //Initialize epoll
    let mut poll = Poll::new().expect("An error occured while initializing the epoll");
    let mut events = Events::with_capacity(1024);

    //Register the server
    poll.registry().register(&mut server, SERVER_TOKEN, Interest::READABLE);

    //Client list
    let mut login_clients: HashMap<Token, PlayerLoginClient> = HashMap::new();
    let mut play_clients: HashMap<Token, PlayerClient> = HashMap::new();
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
                                if login_clients.values().into_iter().any(|x: &PlayerLoginClient| client.1.ip().eq(&x.connection.addr.ip())) {
                                    //TODO Disconnect client
                                    println!("ih");
                                    client.0.shutdown(Shutdown::Both);
                                    continue;
                                }

                                let mut login_client = PlayerLoginClient {
                                    connection: Connection { token: Token(token_counter), stream: client.0, addr: client.1, identifier: client.1.ip().to_string() },
                                    state: ConnectionState::Handshaking,
                                    nickname: None,
                                    verify_token: None,
                                    encode: None,
                                    decode: None,
                                    profile_uuid: None
                                };

                                //Register client in poll and clients vector
                                poll.registry().register(&mut login_client.connection.stream, login_client.connection.token, Interest::READABLE);
                                login_clients.insert(login_client.connection.token, login_client);
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
                    println!("{}", login_clients.len());
                    let mut login_client = login_clients.get_mut(&token);
                    let mut play_client = if login_client.is_none() {play_clients.get_mut(&token)} else {None};

                    //Try to get the connection field from both, but only one is valid
                    let mut connection = match login_client
                        .as_mut().map(|x| &mut x.connection)
                        .xor(play_client.as_mut().map(|mut x| &mut x.connection)) {
                        Some(t) => t,
                        None => {
                            println!("Token not found in epoll event: {}", token.0);
                            continue;
                        }
                    };

                    let mut disconnect = false;

                    //Check for connection states first, this may not trigger in some platforms,
                    //thats why we still keep track on EOF and read 0 while reading the stream
                    if event.is_read_closed() {disconnect = true}
                    else if event.is_error() {
                        disconnect = true;
                        println!("An error occured in client {} socket, told by the epoll", connection.identifier)
                    }

                    //Read values in buffer and copy to vector
                    //first_read to check if, in the first read, the result is 0, meaning that the client has disconnected
                    let mut first_read = true;
                    let mut vec = Vec::with_capacity(2048);

                    if !disconnect {
                        loop {
                            let read = match connection.stream.read(&mut buffer) {
                                Ok(0) if first_read => {
                                    disconnect = true;
                                    break;
                                }
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
                                    println!("An error occured while reading {}'s stream: {}", connection.identifier, e);
                                    disconnect = true;
                                    break;
                                }
                            };

                            first_read = false;
                            vec.extend_from_slice(&buffer[0..read]);
                        }
                    }

                    if disconnect {
                        //TODO Disconnect
                        poll.registry().deregister(&mut connection.stream);
                        if play_client.is_some() {play_clients.remove(&token);}
                        else {login_clients.remove(&token);}
                        break;
                    }

                    let raw_packets = match read_packets(&vec) {
                        Some(t) => t,
                        None => { println!("Error while decoding client {} packets", connection.identifier); continue; }
                    };

                    match login_client {
                        Some(client) => login_handler::handle(raw_packets, client),
                        _ => {}
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
        if index >= data.len() {
            println!("Index bigger 1 {} {} {:?}", index, data.len(), data);
            return None
        };
        let mut length = read_varint(&data[index..], &mut index)? as usize;
        let mut id_length = 0usize;

        //Check if it has no space for id length reading
        if index >= data.len() {
            println!("Index bigger 2 {} {} {:?}", index, data.len(), data);
            return None
        };
        let id = read_varint(&data[index..], &mut id_length)?;
        index += id_length as usize;
        length -= id_length;

        //Check if it has no space for reading the rest of the packet
        if index + length > data.len() {
            println!("Bigger {} {} {} {:?}", index, length, data.len(), data);
            return None
        };
        raw_packets.push(RawPacket {id, data: &data[index..index + length]});
        index += length;
    }

    Some(raw_packets)
}