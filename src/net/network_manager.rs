use crate::data_reader::DataReader;
use crate::data_writer::DataWriter;
use crate::game::chat::ChatComponent;
use crate::game::packets::{ExtendedPacket, Packet};
use crate::net::login_handler;
use crate::net::login_handler::HandleResult;
use crate::net::network_manager::DisconnectReason::{IOError, Timeout};
use aes::cipher::StreamCipher;
use aes::Aes128;
use cfb8::Cfb8;
use mio::event::Source;
use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};
use openssl::rsa::Rsa;
use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{Shutdown, SocketAddr};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::Arc;
use std::time::{Duration, Instant};
use regex::internal::Inst;
use uuid::Uuid;

//Server address
const ADDR: &str = "127.0.0.1:25565";

//Token for epoll identification
const SERVER_TOKEN: Token = Token(0);

const BUFFER_SIZE: usize = 4096;

pub struct Connection {
    pub token: Token,
    pub stream: TcpStream,
    pub addr: SocketAddr,
    pub identifier: String,
}

pub struct PlayerLoginClient {
    pub connection: Connection,
    pub state: ConnectionState,
    pub nickname: Option<String>,
    pub verify_token: Option<[u8; 4]>,
    pub encode: Option<Cfb8<Aes128>>,
    pub decode: Option<Cfb8<Aes128>>,
    pub uuid: Option<Uuid>,
}

impl PlayerLoginClient {
    pub fn write(&mut self, packet: Packet) {
        //Serialize
        let mut data = match packet.serialize_length() {
            Some(t) => t,
            None => return,
        };
        //Encrypt
        match &mut self.encode {
            Some(encode) => encode.encrypt(&mut data),
            None => {}
        }
        //Write
        self.connection.stream.write(&data);
    }

    pub fn write_dc(&mut self, reason: String) {
        //Serialize
        let packet = Packet::DisconnectLogin {
            reason: ChatComponent::new_text(reason),
        };
        let mut serialized = match packet.serialize_length() {
            Some(t) => t,
            None => return,
        };
        //Write
        self.connection.stream.write(&serialized);
    }

    pub fn shutdown(&mut self, reason: String, poll: &Poll) {
        if let ConnectionState::Status = self.state {
            self.write_dc(reason);
        }
        self.connection.stream.flush();
        poll.registry().deregister(&mut self.connection.stream);
        self.connection.stream.shutdown(Shutdown::Both);
    }
}

pub struct PlayerClient {
    connection: Connection,
    encode: Cfb8<Aes128>,
    decode: Cfb8<Aes128>,
    keep_alive: Instant,
}

impl PlayerClient {
    pub fn write(&mut self, packet: Packet) {
        //Serialize
        let mut data = match packet.serialize_length() {
            Some(t) => t,
            None => return,
        };
        //Encrypt
        self.encode.encrypt(&mut data);
        //Write
        self.connection.stream.write(&data);
        self.connection.stream.flush();
    }

    pub fn write_data_ref(&mut self, data: &Vec<u8>) {
        let mut data = data.clone();
        //Encrypt
        self.encode.encrypt(&mut data);
        //Write
        self.connection.stream.write(&data);
    }

    pub fn write_data_owned(&mut self, mut data: Vec<u8>) {
        //Encrypt
        self.encode.encrypt(&mut data);
        //Write
        self.connection.stream.write(&data);
    }

    pub fn write_mut_slice(&mut self, data: &mut [u8]) {
        //Encrypt
        self.encode.encrypt(data);
        //Write
        self.connection.stream.write(&data);
    }

    pub fn shutdown(&mut self, reason: String, poll: &Poll) {
        self.write(Packet::DisconnectPlay {
            reason: ChatComponent::new_text(reason),
        });
        self.shutdown_connection(poll);
    }

    pub fn shutdown_connection(&mut self, poll: &Poll) {
        self.connection.stream.flush();
        poll.registry().deregister(&mut self.connection.stream);
        self.connection.stream.shutdown(Shutdown::Both);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum ConnectionState {
    Handshaking,
    Status,
    Login,
    Play,
}

pub fn start(net_writer: Sender<GameProtocol>, net_reader: Receiver<NetProtocol>) {
    //Open server
    let mut server = TcpListener::bind(ADDR.parse().unwrap())
        .expect("An error occured while binding the server");

    //Initialize epoll
    let mut poll = Poll::new().expect("An error occured while initializing the epoll");
    let mut events = Events::with_capacity(1024);

    //Register the server
    poll.registry()
        .register(&mut server, SERVER_TOKEN, Interest::READABLE);

    //Client list
    let mut login_clients: HashMap<Token, PlayerLoginClient> = HashMap::new();
    let mut play_clients: HashMap<Token, PlayerClient> = HashMap::new();
    let mut token_counter = 1usize;

    println!("Waiting for connections on {}", ADDR);

    std::thread::Builder::new()
        .name("IO Network Thread".to_string())
        .spawn(move || {
            let mut buffer = [0u8; BUFFER_SIZE];
            unsafe {
                let rsa = Rsa::generate(1024).expect("Couldn't generate RSA server key");
                login_handler::PUBLIC_KEY = Some(
                    rsa.public_key_to_der()
                        .expect("Couldn't generate RSA server public key"),
                );
                login_handler::RSA = Some(rsa);
            }

            let mut last_keepalive = Instant::now();

            loop {
                //Poll events
                poll.poll(&mut events, Some(Duration::from_millis(1)));

                let now = Instant::now();

                let send_keepalive = now.duration_since(last_keepalive).as_secs() >= 3;

                for event in events.iter() {
                    let token = event.token();

                    //Check server event
                    if token == SERVER_TOKEN {
                        //Loop to accept all clients and finish server reading
                        loop {
                            match server.accept() {
                                //Got a client
                                Ok(mut client) => {
                                    let mut login_client = PlayerLoginClient {
                                        connection: Connection {
                                            token: Token(token_counter),
                                            stream: client.0,
                                            addr: client.1,
                                            identifier: client.1.ip().to_string(),
                                        },
                                        state: ConnectionState::Handshaking,
                                        nickname: None,
                                        verify_token: None,
                                        encode: None,
                                        decode: None,
                                        uuid: None,
                                    };

                                    //Check if client is already logging
                                    if login_clients.values().into_iter().any(
                                        |x: &PlayerLoginClient| {
                                            login_client
                                                .connection
                                                .addr
                                                .ip()
                                                .eq(&x.connection.addr.ip())
                                        },
                                    ) {
                                        login_client.shutdown(
                                            "One client logging in per time!".to_string(),
                                            &poll,
                                        );
                                        continue;
                                    }

                                    //Register client in poll and clients vector
                                    poll.registry().register(
                                        &mut login_client.connection.stream,
                                        login_client.connection.token,
                                        Interest::READABLE,
                                    );
                                    login_clients.insert(login_client.connection.token, login_client);
                                    token_counter += 1;
                                }

                                //Check if it reached the end
                                Err(ref err) if err.kind() == ErrorKind::WouldBlock => break,

                                //Check for another error
                                Err(e) => {
                                    println!("An error occured while accepting a client: {}", e)
                                }
                            }
                        }
                    } else {
                        //Check for clients token
                        let mut login_client = login_clients.get_mut(&token);
                        let mut play_client = if login_client.is_none() {
                            play_clients.get_mut(&token)
                        } else {
                            None
                        };

                        //Try to get the connection field from both, but only one is valid
                        let mut connection = match login_client
                            .as_mut()
                            .map(|x| &mut x.connection)
                            .xor(play_client.as_mut().map(|mut x| &mut x.connection))
                        {
                            Some(t) => t,
                            None => {
                                println!("Token not found in epoll event: {}", token.0);
                                continue;
                            }
                        };

                        let mut disconnect = false;

                        //Check for connection states first, this may not trigger in some platforms,
                        //thats why we still keep track on EOF and read 0 while reading the stream
                        if event.is_read_closed() {
                            disconnect = true
                        } else if event.is_error() {
                            disconnect = true;
                            println!(
                                "An error occured in client {} socket, told by the epoll",
                                connection.identifier
                            )
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
                                    }

                                    //Check another error
                                    Err(e) => {
                                        println!(
                                            "An error occured while reading {}'s stream: {}",
                                            connection.identifier, e
                                        );
                                        disconnect = true;
                                        break;
                                    }
                                };

                                first_read = false;
                                vec.extend_from_slice(&buffer[0..read]);
                            }
                        }

                        if disconnect {
                            if let Some(client) = play_client {
                                client.shutdown("IO Error".to_string(), &poll);
                                net_writer.send(GameProtocol::ForcedDisconnect {
                                    token,
                                    reason: IOError,
                                });
                                play_clients.remove(&token);
                            } else {
                                login_client
                                    .unwrap()
                                    .shutdown("IO Error".to_string(), &poll);
                                login_clients.remove(&token);
                            }
                            continue;
                        }

                        //If it is play client, then decrypt the data first
                        if let Some(ref mut client) = play_client {
                            client.decode.decrypt(&mut vec);
                        }

                        //Read packets length, id and separe them
                        let raw_packets = match read_packets(&vec) {
                            Some(t) => t,
                            None => {
                                continue;
                            }
                        };

                        //Handle the login
                        match login_client {
                            Some(client) => {
                                let result = login_handler::handle(raw_packets, client);
                                match result {
                                    HandleResult::Disconnect(reason) => {
                                        client.shutdown(reason.to_string(), &poll);
                                        login_clients.remove(&token);
                                        break;
                                    }
                                    HandleResult::Login => {
                                        //Player is ready to go to Play connection state
                                        let client = login_clients.remove(&token).unwrap();

                                        let play_client = PlayerClient {
                                            connection: client.connection,
                                            encode: client.encode.unwrap(),
                                            decode: client.decode.unwrap(),
                                            keep_alive: now,
                                        };

                                        play_clients
                                            .insert(play_client.connection.token, play_client);
                                        net_writer.send(GameProtocol::Login {
                                            token,
                                            uuid: client.uuid.unwrap(),
                                            nickname: client.nickname.unwrap(),
                                        });
                                    }
                                    HandleResult::None => {}
                                }
                                continue;
                            }
                            _ => {}
                        }

                        //Read packets
                        match play_client {
                            Some(player) => {
                                for raw_packet in raw_packets {
                                    let packet = Packet::read(
                                        raw_packet.id,
                                        &mut DataReader::new(raw_packet.data),
                                        ConnectionState::Play,
                                    );
                                    match packet {
                                        Some(packet) => {
                                            //Send packets to be processed by the tick thread
                                            match packet {
                                                Packet::KeepAlive { id } => player.keep_alive = now,
                                                _ => {
                                                    net_writer.send(GameProtocol::Packet {
                                                        token,
                                                        packet,
                                                    });
                                                }
                                            };
                                        }
                                        None => {}
                                    }
                                }
                            }
                            None => {}
                        }
                    }
                }

                //Starts to read the game messages
                for message in net_reader.try_iter() {
                    match message {
                        NetProtocol::SendPacket { token, packet } => {
                            let client = match play_clients.get_mut(&token) {
                                Some(t) => t,
                                None => continue,
                            };
                            client.write(packet);
                        }
                        NetProtocol::SendExtendedPacket { token, mut packet } => {
                            let mut client = match play_clients.get_mut(&token) {
                                Some(t) => t,
                                None => continue,
                            };

                            packet.send(&mut client);
                        }
                        NetProtocol::SendData { token, packet } => {
                            let client = match play_clients.get_mut(&token) {
                                Some(t) => t,
                                None => continue,
                            };
                            client.write_data_ref(&packet);
                        }
                        NetProtocol::SendOwnedData { token, data } => {
                            let client = match play_clients.get_mut(&token) {
                                Some(t) => t,
                                None => continue,
                            };
                            client.write_data_owned(data);
                        }
                        NetProtocol::Unregister { token } => {
                            let client = match play_clients.get_mut(&token) {
                                Some(t) => t,
                                None => continue,
                            };
                            client.shutdown_connection(&poll);
                            play_clients.remove(&token);
                        }
                    }
                }

                if send_keepalive {
                    last_keepalive = now;

                    //Check for clients who are taking too long to send the keep alive
                    play_clients.retain(|token, player| {
                        if now.duration_since(player.keep_alive).as_secs() >= 10 {
                            //Timeout disconnect
                            player.shutdown("Timeout".to_string(), &poll);
                            net_writer.send(GameProtocol::ForcedDisconnect {
                                token: *token,
                                reason: Timeout,
                            });
                            return false;
                        }

                        true
                    });

                    //Send keep alive packets
                    let keep_alive = Packet::KeepAlive { id: 0 }.serialize_length().unwrap();
                    for player in play_clients.values_mut() {
                        player.write_data_ref(&keep_alive);
                    }
                }
            }
        });
}

pub enum NetProtocol {
    SendPacket { token: Token, packet: Packet },
    SendExtendedPacket { token: Token, packet: ExtendedPacket },
    SendData { token: Token, packet: Arc<Vec<u8>> },
    SendOwnedData { token: Token, data: Vec<u8> },
    Unregister { token: Token },
}

pub enum GameProtocol {
    Login {
        token: Token,
        nickname: String,
        uuid: Uuid,
    },
    ForcedDisconnect {
        token: Token,
        reason: DisconnectReason,
    },
    Packet {
        token: Token,
        packet: Packet,
    },
}

pub enum DisconnectReason {
    Timeout,
    IOError,
}

pub struct NetWriter {
    pub writer: Sender<NetProtocol>,
}

impl NetWriter {
    pub fn send_packet(&self, token: Token, packet: Packet) {
        self.writer.send(NetProtocol::SendPacket { token, packet });
    }

    pub fn send_extended_packet(&self, token: Token, packet: ExtendedPacket) {
        self.writer.send(NetProtocol::SendExtendedPacket {token, packet});
    }

    pub fn send_data(&self, token: Token, data: Arc<Vec<u8>>) {
        self.writer.send(NetProtocol::SendData {
            token,
            packet: data,
        });
    }

    pub fn disconnect(&self, token: Token, reason: ChatComponent) {
        self.writer.send(NetProtocol::SendPacket {
            token,
            packet: Packet::DisconnectPlay { reason },
        });
        self.writer.send(NetProtocol::Unregister { token });
    }
}

impl Clone for NetWriter {
    fn clone(&self) -> Self {
        NetWriter {
            writer: self.writer.clone(),
        }
    }
}

pub struct RawPacket<'a> {
    pub id: i32,
    pub data: &'a [u8],
}

fn read_varint<'a>(slice: &[u8], index: &mut usize) -> Option<i32> {
    let mut result: i32 = 0;
    let mut read: u8;
    for i in 0..5 {
        read = *slice.get(i)?;
        result += (((read as i8) & 0b01111111) as i32) << (7 * i);
        *index += 1;

        if (read & 0b10000000) == 0 {
            return Some(result);
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
            return None;
        };
        let mut length = read_varint(&data[index..], &mut index)? as usize;
        let mut id_length = 0usize;

        //Check if it has no space for id length reading
        if index >= data.len() {
            println!("Index bigger 2 {} {} {:?}", index, data.len(), data);
            return None;
        };
        let id = read_varint(&data[index..], &mut id_length)?;
        index += id_length as usize;
        length -= id_length;

        //Check if it has no space for reading the rest of the packet
        if index + length > data.len() {
            println!("Bigger {} {} {} {:?}", index, length, data.len(), data);
            return None;
        };
        raw_packets.push(RawPacket {
            id,
            data: &data[index..index + length],
        });
        index += length;
    }

    Some(raw_packets)
}
