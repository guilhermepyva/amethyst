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
use openssl::rsa::{Rsa, Padding};
use openssl::pkey::Private;
use openssl::rand::rand_bytes;
use rand::Rng;
use crate::packet::encryption_response::PacketEncryptionResponse;
use crate::utils::arrays::extract_vector;
use cfb8::Cfb8;
use aes::Aes128;
use uuid::Uuid;
use std::sync::Arc;
use aes::cipher::{StreamCipher, NewStreamCipher};

pub enum PacketResult {
    Ok(Packet),
    Disconnect(String)
}

lazy_static!(
    static ref RSA: Rsa<Private> = Rsa::generate(1024).unwrap();
);

pub fn handle(packet: Packet, client: &mut LoggingInClient, stream: &mut TcpStream) {
    match packet {
        Packet::Handshake(handshake) => {
            client.state = match handshake.next_state {
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
            client.next_packet = Some(packet::status_response::StatusResponsePacket { json }.write())
        }
        Packet::Ping(ping) => client.next_packet = Some(packet::pong::PongPacket{ pong: ping.ping }.write()),
        Packet::LoginStart(login) => {
            client.nickname = Some(login.nickname);
            let public_key = RSA.public_key_to_der().unwrap();
            let verify_token = rand::thread_rng().gen::<[u8; 4]>().to_vec();

            client.verify_token = Some(verify_token.clone());
            client.next_packet = Some(packet::encryption_request::PacketEncryptionRequest {
                server: String::new(),
                public_key_length: public_key.len() as u32,
                public_key,
                verify_token_length: 4,
                verify_token
            }.write())
        }
        Packet::EncryptionResponse(response) => {
            let mut decrypted_verify_token = [0 as u8; 128];
            match RSA.private_decrypt(&response.verify_token, &mut decrypted_verify_token, Padding::PKCS1) {
                Ok(t) => {},
                Err(_e) => {
                    client.disconnect(stream, "Couldn't decrypt verify token".to_owned());
                    return;
                }
            };
            if !extract_vector(&decrypted_verify_token, 0, 4).eq(client.verify_token.as_ref().unwrap()) {
                client.disconnect(stream, "Verify token isn't the same".to_owned());
                return;
            }

            let mut decrypted_shared_secret = [0 as u8; 128];
            let shared_secret_length = match RSA.private_decrypt(&response.shared_secret, &mut decrypted_shared_secret, Padding::PKCS1) {
                Ok(t) => t,
                Err(_e) => {
                    client.disconnect(stream, "Couldn't decrypt shared secret".to_owned());
                    return;
                }
            };
            let shared_secret = extract_vector(&decrypted_shared_secret, 0, shared_secret_length);
            let mut cfb8 = Cfb8::<Aes128>::new_var(&shared_secret, &shared_secret).unwrap();
            let mut data = packet::login_success::LoginSuccess {
                uuid: Uuid::new_v4(),
                nickname: client.nickname.as_ref().unwrap().clone()
            }.write();
            println!("{:?}", data);
            cfb8.encrypt(&mut data);
            client.next_packet = Some(data);
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