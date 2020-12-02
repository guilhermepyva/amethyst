use crate::packet::WritePacket;
use crate::data_writer::DataWriter;
use json::JsonValue;

pub struct StatusResponsePacket {
    pub json: JsonValue
}
impl WritePacket for StatusResponsePacket {
    fn write(&self) -> Vec<u8> {
        let mut writer = DataWriter::new();

        writer.write_u8(0x00);
        writer.write_string(&self.json.to_string());
        writer.set_lenght(writer.data.len() as u32);

        writer.data
    }
}