use bytes::BufMut;
use crate::game::position::Position;
use rustc_serialize::Encodable;

pub struct DataWriter {
    pub data: Vec<u8>
}

impl DataWriter {
    pub fn new() -> DataWriter {
        DataWriter { data: Vec::new() }
    }

    pub fn write_string(&mut self, string: &String) {
        self.write_varint(string.len() as i32);
        self.data.append(&mut string.as_bytes().to_vec());
    }

    pub fn write_varint(&mut self, mut value: i32) {
        loop {
            let mut temp = (value & 0b01111111) as u8;
            value >>= 7;
            if value != 0 {
                temp |= 0b10000000;
            }
            self.data.push(temp);

            if value == 0 {
                break
            }
        }
    }

    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value)
    }

    pub fn write_i8(&mut self, value: i8) {
        self.data.push(value as u8)
    }

    pub fn write_i32(&mut self, value: i32) {
        self.data.put_i32(value);
    }

    pub fn write_i64(&mut self, value: i64) {
        self.data.put_i64(value);
    }

    pub fn write_bool(&mut self, value: bool) {
        self.data.push(if value {0x01} else {0x00})
    }

    pub fn write_data(&mut self, data: &Vec<u8>) {
        self.data.append(&mut data.clone());
    }

    pub fn write_position(&mut self, position: &Position) {
        self.data.put_i64(position.encode());
    }

    pub fn get_varint(mut value: u32) -> Vec<u8> {
        let mut data = vec![];

        loop {
            let mut temp = (value & 0b01111111) as u8;
            value >>= 7;
            if value != 0 {
                temp |= 0b10000000;
            }
            data.push(temp);

            if value == 0 {
                break
            }
        }

        data
    }
}