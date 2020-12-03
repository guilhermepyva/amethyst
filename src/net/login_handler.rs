use crate::net::network_manager::{RawPacket, LoggingInClient, ConnectionState};
use crate::packet::{Packet, ReadPacket, WritePacket};
use crate::packet::handshake;
use crate::data_reader::DataReader;
use crate::packet;
use json::JsonValue;
use json::number::Number;
use crate::game::chat::ChatComponent;
use std::net::TcpStream;
use crate::packet::login_start::PacketLoginStart;
use lazy_static::lazy_static;
use openssl::rsa::Rsa;
use openssl::pkey::Private;
use openssl::rand::rand_bytes;
use rand::Rng;
use crate::packet::encryption_response::PacketEncryptionResponse;

pub enum PacketResult {
    Ok(Packet),
    Disconnect(String)
}

lazy_static!(
    static ref RSA: Rsa<Private> = Rsa::generate(1024).unwrap();
);

pub fn handle(packet: Packet, client: &LoggingInClient, stream: &mut TcpStream) {
    match packet {
        Packet::Handshake(handshake) => {
            *client.state.lock().unwrap() = match handshake.next_state {
                1 => ConnectionState::Status,
                2 => ConnectionState::Login,
                _ => {
                    client.disconnect(stream, "Unknown state on handshake.".to_owned());
                    return;
                }
            };
        }
        Packet::StatusRequest => {
            let mut json = JsonValue::new_object();
            let mut version = JsonValue::new_object();
            version["name"] = JsonValue::String("1.8.9".to_owned());
            version["protocol"] = JsonValue::Number(Number::from(47 as u8));
            json["version"] = version;
            let mut players = JsonValue::new_object();
            players["max"] = JsonValue::Number(Number::from(10 as u8));
            players["online"] = JsonValue::Number(Number::from(0 as u8));
            json["players"] = players;
            json["description"] = ChatComponent::new_text("Servidor de Minecraft Amethyst".to_owned()).to_json();
            *client.next_packet.lock().unwrap() = Some(packet::status_response::StatusResponsePacket { json }.write())
        }
        Packet::Ping(ping) => *client.next_packet.lock().unwrap() = Some(packet::pong::PongPacket{ pong: ping.ping }.write()),
        Packet::LoginStart(login) => {
            let public_key = RSA.public_key_to_der().unwrap();
            let verify_token = rand::thread_rng().gen::<[u8; 4]>();

            *client.next_packet.lock().unwrap() = Some(packet::encryption_request::PacketEncryptionRequest {
                server: String::new(),
                public_key_length: public_key.len() as u32,
                public_key,
                verify_token_length: 4,
                verify_token: verify_token.to_vec()
            }.write())
        }
        Packet::EncryptionResponse(response) => {
            println!("{}", response.shared_secret_length);
            println!("{}", response.shared_secret.len());
            println!("{}", response.verify_token_length);
            println!("{}", response.verify_token.len());
        }
        _ => {}
    }
}

pub fn get_packet<'a>(packet: RawPacket, state: ConnectionState) -> PacketResult {
    match state {
        ConnectionState::Handshaking => {
            match packet.id {
                0 => {
                    match handshake::PacketHandshake::read(DataReader::new(&packet.data)) {
                        Ok(t) => PacketResult::Ok(t),
                        Err(_e) => PacketResult::Disconnect("Invalid handshake packet.".to_owned())
                    }
                }
                _ => PacketResult::Disconnect("You were supposed to send the handshake packet.".to_owned())
            }
        }
        ConnectionState::Status => {
            match packet.id {
                0x00 => PacketResult::Ok(Packet::StatusRequest),
                0x01 => match packet::ping::PingPacket::read(DataReader::new(&packet.data)) {
                    Ok(t) => PacketResult::Ok(t),
                    Err(_e) => PacketResult::Disconnect("Invalid ping packet.".to_owned())
                },
                _ => PacketResult::Disconnect(format!("Packet id {} doesn't exist in this state", packet.id))
            }
        }
        ConnectionState::Login => {
            match packet.id {
                0x00 => match PacketLoginStart::read(DataReader::new(&packet.data)) {
                    Ok(t) => PacketResult::Ok(t),
                    Err(_e) => PacketResult::Disconnect("Invalid login packet.".to_owned())
                }
                0x01 => match PacketEncryptionResponse::read(DataReader::new(&packet.data)) {
                    Ok(t) => PacketResult::Ok(t),
                    Err(_e) => PacketResult::Disconnect("Invalid encryption response packet.".to_owned())
                }
                _ => PacketResult::Disconnect(format!("Packet id {} doesn't exist in this state", packet.id))
            }
        }
        _ => PacketResult::Disconnect("Not programmed".to_owned())
    }
}