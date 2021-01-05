use crate::packet::WritePacket;
use crate::data_writer::DataWriter;

#[derive(Debug)]
pub struct PongPacket {
    pub pong: i64
}
impl WritePacket for PongPacket {
    fn write(&self) -> Vec<u8> {
        let mut writer = DataWriter::new();

        writer.write_u8(0x01);
        writer.write_i64(self.pong);

        writer.data
    }
}