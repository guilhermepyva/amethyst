use crate::packet::{ReadPacket, Packet};
use crate::data_reader::DataReader;

#[derive(Debug)]
pub struct PacketEncryptionResponse {
    pub shared_secret_length: u32,
    pub shared_secret: Vec<u8>,
    pub verify_token_length: u32,
    pub verify_token: Vec<u8>
}

impl ReadPacket for PacketEncryptionResponse {
    fn read<'a>(mut reader: DataReader) -> Result<Packet, &'a str> {
        let shared_secret_length = reader.read_varint()?;
        let shared_secret = reader.read_data_fixed(shared_secret_length as usize)?;
        let verify_token_length = reader.read_varint()?;

        Ok(Packet::EncryptionResponse(PacketEncryptionResponse{
            shared_secret_length,
            shared_secret,
            verify_token_length,
            verify_token: reader.read_data_fixed(verify_token_length as usize)?
        }))
    }
}