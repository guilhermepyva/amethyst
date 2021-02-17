use crate::data_reader::DataReader;
use crate::net::network_manager::ConnectionState;
use crate::data_writer::DataWriter;
use crate::game::chat::ChatComponent;
use json::JsonValue;
use uuid::Uuid;

#[derive(Debug)]
pub enum Packet{
    Handshake{
        protocol_version: i32,
        server_address: String,
        server_port: u16,
        next_state: u8
    },
    StatusRequest,
    Ping { ping: i64 },
    LoginStart { nickname: String },
    EncryptionRequest {
        server: String,
        public_key_length: i32,
        public_key: Vec<u8>,
        verify_token_length: i32,
        verify_token: Vec<u8>
    },
    DisconnectLogin {
        reason: ChatComponent
    },
    DisconnectPlay {
        reason: ChatComponent
    },
    StatusResponse {
        json: JsonValue
    },
    Pong {
        pong: i64
    },
    EncryptionResponse {
        shared_secret_length: i32,
        shared_secret: Vec<u8>,
        verify_token_length: i32,
        verify_token: Vec<u8>
    },
    LoginSuccess {
        uuid: Uuid,
        nickname: String
    }
}

impl Packet {
    pub fn read<'a>(id: i32, reader: &mut DataReader, state: ConnectionState) -> Result<Packet, &'a str> {
        match state {
            ConnectionState::Handshaking => {
                match id {
                    0x00 => Ok(Packet::Handshake {
                            protocol_version: reader.read_varint()?,
                            server_address: reader.read_string()?,
                            server_port: reader.read_u16()?,
                            next_state: reader.read_u8()?, }),
                    _ => Err("You were supposed to send the handshake packet.")
                }
            }
            ConnectionState::Status => {
                match id {
                    0x00 => Ok(Packet::StatusRequest),
                    0x01 => Ok(Packet::Ping { ping: reader.read_i64()? }),
                    _ => Err("Inexistent packet ID")
                }
            }
            ConnectionState::Login => {
                match id {
                    0x00 => Ok(Packet::LoginStart { nickname: reader.read_string()? }),
                    0x01 => {
                        let shared_secret_length = reader.read_varint()?;
                        let shared_secret = reader.read_data_fixed(shared_secret_length as usize)?;
                        let verify_token_length = reader.read_varint()?;

                        Ok(Packet::EncryptionResponse {
                            shared_secret_length,
                            shared_secret,
                            verify_token_length,
                            verify_token: reader.read_data_fixed(verify_token_length as usize)?
                        })
                    }
                    _ => Err("Inexistent packet ID")
                }
            }
            ConnectionState::Play => {
                match id {
                    _ => Err("Inexistent packet ID")
                }
            }
        }
    }

    pub fn serialize<'a>(&self) -> Result<Vec<u8>, &'a str> {
        let mut writer = DataWriter::new();
        match self {
            Packet::EncryptionRequest {
                public_key,
                public_key_length,
                server,
                verify_token,
                verify_token_length
            } => {
                writer.write_u8(0x01);
                writer.write_string(server);
                writer.write_varint(*public_key_length);
                writer.write_data(public_key);
                writer.write_varint(*verify_token_length);
                writer.write_data(verify_token);
            }
            Packet::DisconnectLogin { reason } => {
                writer.write_u8(0x00);
                writer.write_string(&reason.to_string());
            }
            Packet::DisconnectPlay { reason } => {
                writer.write_u8(0x40);
                writer.write_string(&reason.to_string());
            }
            Packet::StatusResponse { json } => {
                writer.write_u8(0x00);
                writer.write_string(&json.to_string());
            }
            Packet::Pong { pong } => {
                writer.write_u8(0x01);
                writer.write_i64(*pong);
            }
            Packet::LoginSuccess { nickname, uuid } => {
                writer.write_u8(0x02);
                writer.write_string(&uuid.to_hyphenated().to_string());
                writer.write_string(nickname)
            }
            _ => return Err("Can't serialize this packet")
        }

        Ok(writer.data)
    }
}