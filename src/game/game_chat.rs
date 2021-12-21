use crate::game::chat::ChatComponent;
use crate::game::engine::SyncEnvironment;
use crate::game::packets::Packet;
use crate::game::player::Player;

pub fn chat_listener(packet: &Packet, player_index: usize, environment: &mut SyncEnvironment) {
    match packet {
        Packet::ClientChatMessage { message } => {
            // let name = environment.players[player_index].nickname.clone();
            // let packet = Packet::ServerChatMessage {
            //     component: ChatComponent::new_text(format!("§d{}: §f{}", name, message)),
            //     pos: 0
            // };
            // for x in environment.players.iter_mut() {
            //     x.connection.send_packet(&packet);
            // }
        }
        _ => {}
    };
}
