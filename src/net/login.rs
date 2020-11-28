use crate ::server;
use crate::packet::{Packet, PacketStruct, WritePacket};
use lazy_static::lazy_static;
use openssl::rsa::Rsa;
use openssl::pkey::Private;
use crate::packet::login_start::PacketLoginStart;
use crate::packet::encryption_request::PacketEncryptionRequest;

lazy_static!(
    static ref RSA: Rsa<Private> = Rsa::generate(1024).unwrap();
);

pub struct LoginPacketListener {}
impl server::PacketListener for LoginPacketListener {
    fn received(&self, packet: &Packet) {
        let public_key = RSA.public_key_to_der().unwrap();

        match packet {
            Packet::LoginStart(_login_start) => {
                let encryption_request = PacketEncryptionRequest{
                    packet: PacketStruct{id:0x01, uuid: None},
                    server: String::new(),
                    public_key_length: public_key.len() as u32,
                    public_key,
                    verify_token_length: 4,
                    verify_token: vec![5, 5, 5, 5]
                };

                server::send_bytes(encryption_request.write());
            }
            _ => {}
        }
    }
}