use crate::net::network_manager::{LoggingInClient, ConnectionState};
use crate::packets::Packet;
use json::JsonValue;
use json::number::Number;
use crate::game::chat::ChatComponent;
use std::net::TcpStream;
use lazy_static::lazy_static;
use openssl::rsa::{Rsa, Padding};
use openssl::pkey::Private;
use rand::Rng;
use crate::utils::arrays::extract_vector;
use cfb8::Cfb8;
use aes::Aes128;
use uuid::Uuid;
use aes::cipher::NewStreamCipher;
use regex::Regex;
use crypto::sha1::Sha1;
use crypto::digest::Digest;
use rustc_serialize::hex::ToHex;

pub enum PacketResult {
    Ok(Packet),
    Disconnect(String)
}

lazy_static!(
    static ref RSA: Rsa<Private> = Rsa::generate(1024).unwrap();
);

pub fn handle(packet: Packet, client: &mut LoggingInClient, stream: &mut TcpStream) {
    match packet {
        Packet::Handshake { protocol_version: _, server_address: _, server_port: _, next_state } => {
            client.state = match next_state {
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
            client.next_packet = Some(Packet::StatusResponse {json}.serialize().unwrap())
        }
        Packet::Ping { ping } => client.next_packet = Some(Packet::Pong {pong: ping}.serialize().unwrap()),
        Packet::LoginStart { nickname } => {
            client.nickname = Some(nickname);
            let public_key = RSA.public_key_to_der().unwrap();
            let verify_token = rand::thread_rng().gen::<[u8; 4]>().to_vec();

            client.verify_token = Some(verify_token.clone());
            client.next_packet = Some(Packet::EncryptionRequest {
                server: String::new(),
                public_key_length: public_key.len() as u32,
                public_key,
                verify_token_length: 4,
                verify_token
            }.serialize().unwrap())
        }
        Packet::EncryptionResponse { shared_secret_length: _, shared_secret, verify_token_length: _, verify_token } => {
            let mut decrypted_verify_token = [0 as u8; 128];
            match RSA.private_decrypt(&verify_token, &mut decrypted_verify_token, Padding::PKCS1) {
                Ok(_t) => {},
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
            let shared_secret_length = match RSA.private_decrypt(&shared_secret, &mut decrypted_shared_secret, Padding::PKCS1) {
                Ok(t) => t,
                Err(_e) => {
                    client.disconnect(stream, "Couldn't decrypt shared secret".to_owned());
                    return;
                }
            };
            let shared_secret = extract_vector(&decrypted_shared_secret, 0, shared_secret_length);
            client.cfb8 = Some(Cfb8::<Aes128>::new_var(&shared_secret, &shared_secret).unwrap());
            client.shared_secret = Some(shared_secret);
            client.next_packet = Some(Packet::LoginSuccess {
                uuid: Uuid::new_v4(),
                nickname: client.nickname.as_ref().unwrap().clone()
            }.serialize().unwrap());
            let mut sha1 = Sha1::new();
            sha1.input(b"");
            sha1.input(&client.shared_secret.as_ref().unwrap());
            sha1.input(&RSA.public_key_to_der().unwrap());

            let test = reqwest::blocking::Client::new();
            let response = test.get(&format!("https://sessionserver.mojang.com/session/minecraft/hasJoined?username={}&serverId={}", client.nickname.as_ref().unwrap(), hex_digest(sha1)))
                .send()
                .unwrap();
            println!("{}", response.text().unwrap())
        }
        _ => {}
    }
}

// pub fn get_packet<'a>(packet: RawPacket, state: ConnectionState) -> PacketResult {
//     match state {
//         ConnectionState::Handshaking => {
//             match packet.id {
//                 0 => {
//                     match handshake::PacketHandshake::read(DataReader::new(&packet.data)) {
//                         Ok(t) => PacketResult::Ok(t),
//                         Err(_e) => PacketResult::Disconnect("Invalid handshake packets.".to_owned())
//                     }
//                 }
//                 _ => PacketResult::Disconnect("You were supposed to send the handshake packets.".to_owned())
//             }
//         }
//         ConnectionState::Status => {
//             match packet.id {
//                 0x00 => PacketResult::Ok(Packet::StatusRequest),
//                 0x01 => match packets::ping::PingPacket::read(DataReader::new(&packet.data)) {
//                     Ok(t) => PacketResult::Ok(t),
//                     Err(_e) => PacketResult::Disconnect("Invalid ping packets.".to_owned())
//                 },
//                 _ => PacketResult::Disconnect(format!("Packet id {} doesn't exist in this state", packet.id))
//             }
//         }
//         ConnectionState::Login => {
//             match packet.id {
//                 0x00 => match PacketLoginStart::read(DataReader::new(&packet.data)) {
//                     Ok(t) => PacketResult::Ok(t),
//                     Err(_e) => PacketResult::Disconnect("Invalid login packets.".to_owned())
//                 }
//                 0x01 => match PacketEncryptionResponse::read(DataReader::new(&packet.data)) {
//                     Ok(t) => PacketResult::Ok(t),
//                     Err(_e) => PacketResult::Disconnect("Invalid encryption response packets.".to_owned())
//                 }
//                 _ => PacketResult::Disconnect(format!("Packet id {} doesn't exist in this state", packet.id))
//             }
//         }
//         _ => PacketResult::Disconnect("Not programmed".to_owned())
//     }
// }

fn hex_digest(mut sha1: Sha1) -> String {
    let mut hex: Vec<u8> = std::iter::repeat(0).take((sha1.output_bits() + 7)/8).collect();
    sha1.result(&mut hex);

    let negative = (hex[0] & 0x80) == 0x80;

    let regex = Regex::new(r#"^0+"#).unwrap();

    if negative {
        two_complement(&mut hex);
        format!("-{}", regex.replace(hex.as_slice().to_hex().as_str(), "").to_string())
    }
    else {
        regex.replace(hex.as_slice().to_hex().as_str(), "").to_string()
    }
}

fn two_complement(bytes: &mut Vec<u8>) {
    let mut carry = true;
    for i in (0..bytes.len()).rev() {
        bytes[i] = !bytes[i] & 0xff;
        if carry {
            carry = bytes[i] == 0xff;
            bytes[i] = bytes[i] + 1;
        }
    }
}