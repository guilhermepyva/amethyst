use std::sync::Arc;
use crate::net::network_manager::MinecraftClient;

#[derive(Debug)]
pub struct PacketStatusRequest {
    pub client: Arc<MinecraftClient>
}