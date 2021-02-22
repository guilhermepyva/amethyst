use crate::data_reader::DataReader;
use crate::net::network_manager::ConnectionState;
use crate::data_writer::DataWriter;
use crate::game::chat::ChatComponent;
use json::JsonValue;
use uuid::Uuid;
use crate::game::position::Position;
use crate::game::packets::Packet::InexistentPacket;

pub enum Packet{
    InexistentPacket,

    Handshake {
        protocol_version: i32,
        server_address: String,
        server_port: u16,
        next_state: u8
    },

    //Status
    StatusRequest,
    Ping {ping: i64},
    StatusResponse {json: JsonValue},
    Pong {pong: i64},

    //Login
    LoginStart {nickname: String},
    EncryptionRequest {
        server: String,
        public_key_length: i32,
        public_key: Vec<u8>,
        verify_token_length: i32,
        verify_token: Vec<u8>
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
    },
    DisconnectLogin {reason: ChatComponent},

    //Play
    KeepAlive {id: i32},
    JoinGame {
        entity_id: i32,
        gamemode: u8,
        dimension: i8,
        difficulty: u8,
        max_players: u8,
        level_type: String,
        reduced_debug_info: bool
    },
    SpawnPosition {location: Position},
    HeldItemChange {slot: u8},
    PlayerInfo {
        action_id: i32,
        players: Vec<PlayerInfoPlayer>
    },
    DisconnectPlay {reason: ChatComponent},
    PlayerPositionAndLook {
        x: f64,
        y: f64,
        z: f64,
        yaw: f32,
        pitch: f32,
        flags: u8
    },
    WorldBorder {
        action: WorldBorderAction
    },
    TimeUpdate {
        world_age: i64,
        time_of_day: i64
    },
    WindowItems {
        window_id: u8,
        slots: Vec<Slot>
    }
}

pub struct Slot {
    present: bool,
    item_id: Option<i32>,
    item_count: Option<i8>,
    //TODO NBT field
}

pub enum WorldBorderAction {
    SetSize {
        radius: f64
    },
    LerpSize {
        old_radius: f64,
        new_radius: f64,
        speed: i32
    },
    SetCenter {
        x: f64,
        z: f64
    },
    Initialize {
        x: f64,
        z: f64,
        old_radius: f64,
        new_radius: f64,
        speed: i64,
        portal_teleport_boundary: i32,
        warning_time: i32,
        warning_blocks: i32
    },
    SetWarningTime {
        warning_time: i32
    },
    SetWarningBlocks {
        warning_blocks: i32
    }
}

pub struct PlayerInfoPlayer {
    pub uuid: Uuid,
    pub action: PlayerInfoAction
}

pub enum PlayerInfoAction {
    AddPlayer {
        name: String,
        properties: Vec<PlayerInfoProperties>,
        gamemode: i32,
        ping: i32,
        display_name: Option<ChatComponent>
    },
    UpdateGameMode {gamemode: i32},
    UpdateLatency {ping: i32},
    UpdateDisplayName {
        display_name: Option<ChatComponent>
    },
    RemovePlayer
}

pub struct PlayerInfoProperties {
    pub name: String,
    pub value: String,
    pub signature: Option<String>
}

impl Packet {
    pub fn read<'a>(id: i32, reader: &mut DataReader, state: ConnectionState) -> Result<Packet, &'a str> {
        match state {
            ConnectionState::Play => {
                match id {
                    0x00 => Ok(Packet::KeepAlive {id: reader.read_varint()?}),
                    _ => Ok(InexistentPacket)
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
            Packet::DisconnectLogin {reason} => {
                writer.write_u8(0x00);
                writer.write_string(&reason.to_string());
            }
            Packet::DisconnectPlay {reason} => {
                writer.write_u8(0x40);
                writer.write_string(&reason.to_string());
            }
            Packet::StatusResponse {json} => {
                writer.write_u8(0x00);
                writer.write_string(&json.to_string());
            }
            Packet::Pong {pong} => {
                writer.write_u8(0x01);
                writer.write_i64(*pong);
            }
            Packet::LoginSuccess {nickname, uuid} => {
                writer.write_u8(0x02);
                writer.write_string(&uuid.to_hyphenated().to_string());
                writer.write_string(nickname)
            }
            Packet::KeepAlive {id} => {
                writer.write_u8(0x00);
                writer.write_varint(*id);
            }
            Packet::JoinGame {
                entity_id,
                gamemode,
                dimension,
                difficulty,
                max_players,
                level_type,
                reduced_debug_info
            } => {
                writer.write_u8(0x01);
                writer.write_i32(*entity_id);
                writer.write_u8(*gamemode);
                writer.write_i8(*dimension);
                writer.write_u8(*difficulty);
                writer.write_u8(*max_players);
                writer.write_string(level_type);
                writer.write_bool(*reduced_debug_info);
            }
            Packet::SpawnPosition {location} => {
                writer.write_u8(0x05);
                writer.write_position(location);
            }
            Packet::HeldItemChange {slot} => {
                writer.write_u8(0x09);
                writer.write_u8(*slot);
            }
            Packet::PlayerInfo {action_id, players} => {
                writer.write_u8(0x38);
                writer.write_varint(*action_id);
                writer.write_varint(players.len() as i32);
                for player in players {
                    writer.write_data(&player.uuid.as_bytes().to_vec());
                    match &player.action {
                        PlayerInfoAction::AddPlayer {
                            name,
                            properties,
                            gamemode,
                            ping,
                            display_name
                        } => {
                            writer.write_string(name);
                            writer.write_varint(properties.len() as i32);
                            for property in properties {
                                writer.write_string(&property.name);
                                writer.write_string(&property.value);
                                if property.signature.is_some() {
                                    writer.write_bool(true);
                                    writer.write_string(property.signature.as_ref().unwrap());
                                } else {
                                    writer.write_bool(false);
                                }
                            }
                            writer.write_varint(*gamemode);
                            writer.write_varint(*ping);
                            if display_name.is_some() {
                                writer.write_bool(true);
                                writer.write_string(&display_name.as_ref().unwrap().to_string());
                            } else {
                                writer.write_bool(false);
                            }
                        }
                        PlayerInfoAction::UpdateGameMode {gamemode} => {
                            writer.write_varint(*gamemode);
                        }
                        PlayerInfoAction::UpdateLatency {ping} => {
                            writer.write_varint(*ping);
                        }
                        PlayerInfoAction::UpdateDisplayName {display_name} => {
                            if display_name.is_some() {
                                writer.write_bool(true);
                                writer.write_string(&display_name.as_ref().unwrap().to_string());
                            } else {
                                writer.write_bool(false);
                            }
                        }
                        PlayerInfoAction::RemovePlayer => {}
                    };
                }
            }
            Packet::PlayerPositionAndLook {
                x,
                y,
                z,
                yaw,
                pitch,
                flags
            } => {
                writer.write_u8(0x08);
                writer.write_f64(*x);
                writer.write_f64(*y);
                writer.write_f64(*z);
                writer.write_f32(*yaw);
                writer.write_f32(*pitch);
                writer.write_u8(*flags);
            },
            Packet::WorldBorder {action} => {
                writer.write_u8(0x44);
                match action {
                    WorldBorderAction::SetSize {radius} => {
                        writer.write_varint(0);
                        writer.write_f64(*radius);
                    }
                    WorldBorderAction::LerpSize {old_radius, new_radius, speed} => {
                        writer.write_varint(1);
                        writer.write_f64(*old_radius);
                        writer.write_f64(*new_radius);
                        writer.write_varint(*speed);
                    }
                    WorldBorderAction::SetCenter {x, z} => {
                        writer.write_varint(2);
                        writer.write_f64(*x);
                        writer.write_f64(*z);
                    }
                    WorldBorderAction::Initialize {
                        x,
                        z,
                        old_radius,
                        new_radius,
                        speed,
                        portal_teleport_boundary,
                        warning_time,
                        warning_blocks
                    } => {
                        writer.write_varint(3);
                        writer.write_f64(*x);
                        writer.write_f64(*z);
                        writer.write_f64(*old_radius);
                        writer.write_f64(*new_radius);
                        writer.write_varlong(*speed);
                        writer.write_varint(*portal_teleport_boundary);
                        writer.write_varint(*warning_time);
                        writer.write_varint(*warning_blocks);
                    }
                    WorldBorderAction::SetWarningTime {warning_time} => {
                        writer.write_varint(4);
                        writer.write_varint(*warning_time);
                    }
                    WorldBorderAction::SetWarningBlocks {warning_blocks} => {
                        writer.write_varint(5);
                        writer.write_varint(*warning_blocks);
                    }
                }
            }
            Packet::TimeUpdate {world_age, time_of_day} => {
                writer.write_varint(0x03);
                writer.write_i64(*world_age);
                writer.write_i64(*time_of_day)
            }
            Packet::WindowItems {window_id, slots} => {
                writer.write_varint(0x30);
                writer.write_u8(*window_id);
                writer.write_i16(slots.len() as i16);
                for slot in slots {
                    writer.write_bool(slot.present);
                    if slot.present {
                        writer.write_varint(slot.item_id.unwrap());
                        writer.write_i8(slot.item_count.unwrap());
                        //TODO implement NBT writing
                    }
                }
            }
            _ => return Err("Can't serialize this packet")
        }

        Ok(writer.data)
    }
}