use uuid::Uuid;
use crate::packet::WritePacket;
use crate::data_writer::DataWriter;

pub struct LoginSuccess {
    pub uuid: Uuid,
    pub nickname: String
}

impl WritePacket for LoginSuccess {
    fn write(&self) -> Vec<u8> {
        let mut writer = DataWriter::new();

        writer.write_u8(0x02);
        writer.write_data(&self.uuid.as_bytes().to_vec());

        writer.data
    }
}