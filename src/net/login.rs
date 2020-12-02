use crate::packet::Packet;
use lazy_static::lazy_static;
use openssl::rsa::Rsa;
use openssl::pkey::Private;
use crate::net::network_manager::{PacketListener, MinecraftClient, ConnectionState};
use std::sync::{Arc, Mutex};

struct LoginSession {
    client: Arc<MinecraftClient>,
    pub protocol_version: u32,
    pub server_address: String,
    pub server_port: u16,
}

lazy_static!(
    static ref RSA: Rsa<Private> = Rsa::generate(1024).unwrap();
    static ref LOGGING_IN: Mutex<Vec<LoginSession>> = Mutex::new(vec![]);
);

pub struct LoginPacketListener {}
impl PacketListener for LoginPacketListener {
    fn received(&self, client: Arc<MinecraftClient>, packet: &Packet) {
        let public_key = RSA.public_key_to_der().unwrap();
        println!("{:?}", packet);

        match packet {
            Packet::Handshake(handshake) => {
                LOGGING_IN.lock().unwrap().push(LoginSession{
                    client: client.clone(),
                    protocol_version: handshake.protocol_version,
                    server_address: handshake.server_address.clone(),
                    server_port: handshake.server_port,
                });

                *client.state.lock().unwrap() = match handshake.next_state {
                    1 => ConnectionState::Status,
                    2 => ConnectionState::Login,
                    _ => {
                        client.disconnect("Unknown state on handshake.".to_owned());
                        return;
                    }
                };
            }
            Packet::LoginStart(_login_start) => {

                // let encryption_request = PacketEncryptionRequest{
                //     packet: PacketStruct{id:0x01, uuid: None},
                //     server: String::new(),
                //     public_key_length: public_key.len() as u32,
                //     public_key,
                //     verify_token_length: 4,
                //     verify_token: vec![5, 5, 5, 5]
                // };

                //server::send_bytes(encryption_request.write());
            }
            _ => {}
        }
    }
}