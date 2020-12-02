use crate::net::network_manager::{PacketListener, ConnectionState, MinecraftClient};
use crate::packet::{Packet, WritePacket};
use crate::packet::status_response::StatusResponsePacket;
use json::JsonValue;
use json::number::Number;
use crate::game::chat::ChatComponent;
use crate::packet::pong::PongPacket;
use std::time::Duration;
use std::sync::Arc;

pub struct StatusPacketListener {}
impl PacketListener for StatusPacketListener {
    fn received(&self, client: Arc<MinecraftClient>, packet: &Packet) {
        match packet {
            Packet::StatusRequest(packet) => {
                match *client.state.lock().unwrap() {
                    ConnectionState::Status => {
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
                        client.send_packet(StatusResponsePacket {json}.write());
                    }
                    _ => {
                        client.disconnect("Connection state isn't in status mode.".to_owned());
                        return;
                    }
                }
            }
            Packet::Ping(packet) => {
                match *client.state.lock().unwrap() {
                    ConnectionState::Status => {
                        client.send_packet(PongPacket {pong: packet.ping}.write())
                    }
                    _ => {
                        client.disconnect("Connection state isn't in status mode.".to_owned());
                        return;
                    }
                }
            }
            _ => {}
        }
    }
}