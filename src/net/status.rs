use crate::net::network_manager::{PacketListener, ConnectionState};
use crate::packet::{Packet, WritePacket};
use crate::packet::status_response::StatusResponsePacket;
use json::JsonValue;
use json::number::Number;
use crate::game::chat::ChatComponent;
use crate::packet::pong::PongPacket;
use std::time::Duration;

pub struct StatusPacketListener {}
impl PacketListener for StatusPacketListener {
    fn received(&self, packet: &Packet) {
        match packet {
            Packet::StatusRequest(packet) => {
                match *packet.client.state.lock().unwrap() {
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
                        packet.client.send_packet(StatusResponsePacket {client: packet.client.clone(), json }.write());
                    }
                    _ => {
                        packet.client.disconnect(packet.client.clone(), "Connection state isn't in status mode.".to_owned());
                        return;
                    }
                }
            }
            Packet::Ping(packet) => {
                match *packet.client.state.lock().unwrap() {
                    ConnectionState::Status => {
                        packet.client.send_packet(PongPacket {client: packet.client.clone(), pong: *packet.client.ping.lock().unwrap()}.write())
                    }
                    _ => {
                        packet.client.disconnect(packet.client.clone(), "Connection state isn't in status mode.".to_owned());
                        return;
                    }
                }
            }
            _ => {}
        }
    }
}