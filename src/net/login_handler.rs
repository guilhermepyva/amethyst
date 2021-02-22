use crate::net::network_manager::{LoggingInClient, ConnectionState};
use crate::game::packets::Packet;
use json::JsonValue;
use json::number::Number;
use crate::game::chat::ChatComponent;
use openssl::rsa::{Rsa, Padding};
use openssl::pkey::Private;
use rand::Rng;
use crate::utils::arrays::extract_vector;
use cfb8::Cfb8;
use aes::Aes128;
use uuid::Uuid;
use aes::cipher::NewStreamCipher;
use regex::Regex;
use rustc_serialize::hex::ToHex;
use openssl::sha::Sha1;
use std::str::FromStr;

pub enum HandleResult<'a> {
    SendPacket(Packet),
    Disconnect(&'a str),
    Nothing
}

pub fn handle<'a>(packet: Packet, client: &mut LoggingInClient) -> HandleResult<'a> {
    return match packet {
        Packet::Handshake { protocol_version: _, server_address: _, server_port: _, next_state } => {
            client.state = match next_state {
                1 => ConnectionState::Status,
                2 => ConnectionState::Login,
                _ => return HandleResult::Disconnect("Unknown state on handshake")
            };
            HandleResult::Nothing
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
            HandleResult::SendPacket(Packet::StatusResponse { json })
        }
        Packet::Ping { ping } => HandleResult::SendPacket(Packet::Pong { pong: ping }),
        Packet::LoginStart { nickname } => {
            client.nickname = Some(nickname);
            let public_key = get_rsa().public_key_to_der().unwrap();
            let verify_token = rand::thread_rng().gen::<[u8; 4]>().to_vec();

            client.verify_token = Some(verify_token.clone());
            HandleResult::SendPacket(Packet::EncryptionRequest {
                server: String::new(),
                public_key_length: public_key.len() as i32,
                public_key,
                verify_token_length: 4,
                verify_token
            })
        }
        Packet::EncryptionResponse { shared_secret_length: _, shared_secret, verify_token_length: _, verify_token } => {
            let rsa = get_rsa();
            let mut decrypted_verify_token = [0 as u8; 128];
            match rsa.private_decrypt(&verify_token, &mut decrypted_verify_token, Padding::PKCS1) {
                Ok(_t) => {},
                Err(_e) => return HandleResult::Disconnect("Couldn't decrypt verify token")
            };
            if !extract_vector(&decrypted_verify_token, 0, 4).eq(client.verify_token.as_ref().unwrap()) {
                return HandleResult::Disconnect("Verify token isn't the same")
            }

            let mut decrypted_shared_secret = [0 as u8; 128];
            let shared_secret_length = match rsa.private_decrypt(&shared_secret, &mut decrypted_shared_secret, Padding::PKCS1) {
                Ok(t) => t,
                Err(_e) => {
                    return HandleResult::Disconnect("Couldn't decrypt shared secret")
                }
            };
            let shared_secret = extract_vector(&decrypted_shared_secret, 0, shared_secret_length);

            client.encode = Some(Cfb8::<Aes128>::new_var(&shared_secret, &shared_secret).unwrap());
            client.decode = Some(Cfb8::<Aes128>::new_var(&shared_secret, &shared_secret).unwrap());

            let mut sha1 = Sha1::new();
            sha1.update(b"");
            sha1.update(&shared_secret);
            sha1.update(&rsa.public_key_to_der().unwrap());

            let response = match reqwest::blocking::Client::new().get(&format!("https://sessionserver.mojang.com/session/minecraft/hasJoined?username={}&serverId={}", client.nickname.as_ref().unwrap(), hex_digest(sha1)))
                .send() {
                Ok(ok) => ok,
                Err(e) => {
                    println!("Error while contacting sessionserver.mojang.com to login a player: {}, {}", client.nickname.as_ref().unwrap(), e);
                    return HandleResult::Disconnect("Couldn't decrypt shared secret")
                }
            };
            let response_code = response.status().as_u16();
            if response_code == 204 {
                return HandleResult::Disconnect("Client not authenticated.")
            }
            let json = match response.text() {
                Ok(ok) => {
                    match json::parse(&ok) {
                        Ok(ok) => ok,
                        Err(e) => {
                            println!("Error while parsing login response to json: {}, {}", client.nickname.as_ref().unwrap(), e);
                            return HandleResult::Disconnect("An error occured while contacting Mojang.")
                        }
                    }
                },
                Err(e) => {
                    println!("Error while parsing login response to text: {}, {}", client.nickname.as_ref().unwrap(), e);
                    return HandleResult::Disconnect("An error occured while contacting Mojang.")
                }
            };

            client.profile_uuid = Some(Uuid::from_str(json["id"].as_str().unwrap()).unwrap());
            HandleResult::SendPacket(Packet::LoginSuccess {
                uuid: Uuid::from_str(json["id"].as_str().unwrap()).unwrap(),
                nickname: json["name"].as_str().unwrap().to_owned()
            })
        }
        _ => HandleResult::Nothing
    }
}

pub static mut RSA: Option<Rsa<Private>> = None;
fn get_rsa() -> &'static Rsa<Private> {
    unsafe {
        return RSA.as_ref().unwrap();
    }
}

fn hex_digest(sha1: Sha1) -> String {
    let mut hex = sha1.finish();

    let negative = (hex[0] & 0x80) == 0x80;

    let regex = Regex::new(r#"^0+"#).unwrap();

    if negative {
        two_complement(&mut hex);
        format!("-{}", regex.replace(&hex.to_hex(), "").to_string())
    }
    else {
        regex.replace(&hex.to_hex(), "").to_string()
    }
}

fn two_complement(bytes: &mut [u8; 20]) {
    let mut carry = true;
    for i in (0..bytes.len()).rev() {
        bytes[i] = !bytes[i] & 0xff;
        if carry {
            carry = bytes[i] == 0xff;
            bytes[i] = bytes[i] + 1;
        }
    }
}