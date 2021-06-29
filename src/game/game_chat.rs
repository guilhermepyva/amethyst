use crate::net::packet_listener::PacketListener;
use crate::game::packets::Packet;
use crate::game::player::Player;
use crate::game::engine::SyncEnvironment;
use crate::game::chat::ChatComponent;

pub struct ChatListener {}

impl PacketListener for ChatListener {
    fn listen(&self, packet: &Packet, player_index: usize, environment: &mut SyncEnvironment) {
        let packet = match packet {
            Packet::ClientChatMessage {message} => {
                let name = environment.players[player_index].nickname.clone();
                let packet = Packet::ServerChatMessage {
                    component: ChatComponent::new_text(format!("§d{}: §f{}", name, message)),
                    pos: 0
                };
                for x in environment.players.iter_mut() {
                    x.connection.send_packet(&packet);
                }
            }
            _ => {}
        };
    }
}