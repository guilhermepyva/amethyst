use crate::utils::arrays;
use std::net::{TcpListener, TcpStream, SocketAddr, Shutdown};
use uuid::Uuid;
use std::sync::{Mutex, Arc};
use io::Read;
use std::io;
use crate::data_reader::DataReader;
use std::io::Write;
use crate::game::chat::ChatComponent;
use crate::net::login_handler;
use cfb8::Cfb8;
use aes::Aes128;
use crate::data_writer::DataWriter;
use aes::cipher::StreamCipher;
use crate::packets::Packet;
use crate::player::{Player, PlayerConnection, PlayerList};
use crate::game::engine::SyncEnvironment;
use login_handler::HandleResult;
use openssl::rsa::Rsa;

#[derive(Debug, Copy, Clone)]
pub enum ConnectionState {
    Handshaking,
    Status,
    Login,
    Play
}

pub struct LoggingInClient {
    pub uuid: Uuid,
    pub addr: SocketAddr,
    pub state: ConnectionState,
    pub nickname: Option<String>,
    pub verify_token: Option<Vec<u8>>,
    pub cfb8: Option<Cfb8<Aes128>>,
    pub profile_uuid: Option<Uuid>
}

pub struct LoggingIn {
    pub uuid: Uuid,
    pub addr: SocketAddr
}

pub type LoggingInList = Arc<Mutex<Vec<LoggingIn>>>;

impl LoggingInClient {
    pub fn disconnect(&mut self, stream: &mut TcpStream, reason: String, logging_in: &mut Vec<LoggingIn>) {
        let mut packet = Packet::DisconnectLogin {reason: ChatComponent::new_text(reason)}.serialize().unwrap();
        packet.splice(0..0, DataWriter::get_varint(packet.len() as u32));
        stream.write(&packet);
        stream.flush();
        stream.shutdown(Shutdown::Both);
        let index = logging_in.iter().position(|x| x.uuid.eq(&self.uuid)).unwrap();
        logging_in.remove(index);
    }
}

#[derive(Clone)]
pub struct RawPacket {
    pub id: i32,
    pub data: Vec<u8>
}

pub fn start(players: PlayerList) {
    let logging_in: LoggingInList = Arc::new(Mutex::new(Vec::new()));
    unsafe {
        login_handler::RSA = Some(Rsa::generate(1024).unwrap());
    }
    let server = match TcpListener::bind("0.0.0.0:25565") {
        Ok(t) => t,
        Err(e) => {
            println!("Error while binding server: {}", e);
            return;
        }
    };
    println!("Aguardando conexões");

    std::thread::Builder::new().name("Amethyst - Client Handler Thread".to_owned()).spawn(move || {
        'outer: loop {
            let (mut stream, addr) = match server.accept() {
                Ok(t) => t,
                Err(e) => {
                    println!("Error while accepting client: {}", e.to_string());
                    continue;
                }
            };

            let mut logging_in_lock = logging_in.lock().unwrap();

            for logging_in in logging_in_lock.iter() {
                if addr.ip().eq(&logging_in.addr.ip()) {
                    let mut packet = Packet::DisconnectLogin { reason: ChatComponent::new_text("Just one client logging in per time!".to_owned()) }.serialize().unwrap();
                    packet.splice(0..0, DataWriter::get_varint(packet.len() as u32));
                    stream.write(&packet);
                    stream.flush();
                    stream.shutdown(Shutdown::Both);
                    continue 'outer;
                }
            }

            let mut client = LoggingInClient{
                uuid: Uuid::new_v4(),
                addr,
                state: ConnectionState::Handshaking,
                nickname: None,
                verify_token: None,
                cfb8: None,
                profile_uuid: None
            };
            logging_in_lock.push(LoggingIn {uuid: client.uuid, addr});

            drop(logging_in_lock);

            let logging_in = logging_in.clone();
            let players = players.clone();
            std::thread::spawn(move || {
                let mut buf: [u8; 512] = [0; 512];
                'outer: loop {
                    let read = match stream.read(&mut buf) {
                        Ok(t) => t,
                        Err(e) => {
                            println!("An error occurred while reading data in Minecraft Client {}: {}", client.addr.ip(),e.to_string());
                            client.disconnect(&mut stream, "An error occurred while reading data from Socket.".to_owned(), &mut logging_in.lock().unwrap());
                            break;
                        }
                    };

                    if read == 0 {
                        let mut logging_in = logging_in.lock().unwrap();
                        let index = match logging_in.iter().position(|x| x.uuid.eq(&client.uuid)) {
                            Some(t) => t,
                            None => break
                        };
                        logging_in.remove(index);
                        break;
                    }

                    let data_vec = arrays::extract_vector(&buf, 0, read);
                    let mut reader = DataReader::new(&data_vec);
                    let packets = match read_packets(&mut reader, read) {
                        Ok(t) => t,
                        Err(_e) => {
                            client.disconnect(&mut stream, "Packets corrupted, closing connection.".to_owned(), &mut logging_in.lock().unwrap());
                            break;
                        }
                    };
                    for raw_packet in packets {
                        let packet = match Packet::read(raw_packet.id, &mut DataReader::new(&raw_packet.data), client.state) {
                            Ok(packet) => packet,
                            Err(string) => {
                                client.disconnect(&mut stream, string.to_owned(), &mut logging_in.lock().unwrap());
                                break 'outer;
                            }
                        };

                        let result = login_handler::handle(packet, &mut client);
                        match result {
                            HandleResult::SendPacket(packet) => {
                                let mut packet_binary = packet.serialize().unwrap();
                                if client.cfb8.is_some() {
                                    client.cfb8.as_mut().unwrap().encrypt(&mut packet_binary);
                                }
                                packet_binary.splice(0..0, DataWriter::get_varint(packet_binary.len() as u32));
                                stream.write(&packet_binary);
                                if let Packet::LoginSuccess{nickname, uuid} = packet {
                                    let mut logging_in = logging_in.lock().unwrap();
                                    let index = logging_in.iter().position(|x| x.uuid.eq(&client.uuid)).unwrap();
                                    let connection = logging_in.remove(index);
                                    drop(logging_in);
                                    players.lock().unwrap().push(Player {
                                        connection: PlayerConnection {
                                            addr: connection.addr,
                                            stream
                                        },
                                        uuid: uuid.clone(),
                                        nickname: nickname.clone(),
                                        encryption: client.cfb8.unwrap()
                                    });
                                    break 'outer;
                                }
                            }
                            HandleResult::Disconnect(message) => {
                                let message = message.to_owned();
                                client.disconnect(&mut stream, message, &mut logging_in.lock().unwrap());
                                break 'outer;
                            }
                            HandleResult::Nothing => {}
                        }
                    }
                }
            });
        }
    }).expect("couldn't open thread");
}

pub fn tick_read_packets(sync_env: SyncEnvironment) {
    
}

fn read_packets<'a>(reader: &mut DataReader, read: usize) -> Result<Vec<RawPacket>, &'a str> {
    let mut vec = vec![];

    while reader.cursor != read {
        let length = reader.read_varint()?;
        let length_length = reader.cursor;
        let id = reader.read_varint()?;

        vec.push(RawPacket {
            id,
            data: reader.read_data_fixed((length as usize) - (reader.cursor - length_length))?
        });
    }

    Ok(vec)
}