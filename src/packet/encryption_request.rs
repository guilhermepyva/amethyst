use crate::packet::{PacketStruct, WritePacket};
use crate::data_writer::DataWriter;

#[derive(Debug)]
pub struct PacketEncryptionRequest {
    pub packet: PacketStruct,
    pub server: String,
    pub public_key_length: u32,
    pub public_key: Vec<u8>,
    pub verify_token_length: u32,
    pub verify_token: Vec<u8>
}

impl WritePacket for PacketEncryptionRequest {
    fn write(&self) -> Vec<u8> {
        let mut writer = DataWriter::new();

        writer.write_u8(0x01);
        writer.write_string(&self.server);
        writer.write_varint(self.public_key_length);
        writer.write_data(&self.public_key);
        writer.write_varint(self.verify_token_length);
        writer.write_data(&self.verify_token);
        writer.set_lenght(writer.data.len() as u32);

        writer.data
    }
}