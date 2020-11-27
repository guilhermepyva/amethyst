use crate::datareader::DataReader;

pub struct RawPacket {
    pub id: u8,
    pub data: Vec<u8>
}

impl RawPacket {
    pub fn get_reader(&self) -> DataReader {
        return DataReader::new(&self.data);
    }
}