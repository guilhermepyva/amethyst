use crate::net::network_manager::MinecraftClient;
use std::sync::Arc;
use crate::packet::{ReadPacket, Packet, WritePacket};
use crate::data_reader::DataReader;
use crate::data_writer::DataWriter;

#[derive(Debug)]
pub struct PongPacket {
    pub client: Arc<MinecraftClient>,
    pub pong: i64
}
impl WritePacket for PongPacket {
    fn write(&self) -> Vec<u8> {
        let mut writer = DataWriter::new();

        writer.write_u8(0x01);
        writer.write_i64(self.pong);
        writer.set_lenght(writer.data.len() as u32);

        writer.data
    }
}