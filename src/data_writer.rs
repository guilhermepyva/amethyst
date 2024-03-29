use crate::game::world::coords::Position;
use arrayvec::ArrayVec;

pub struct DataWriter {
    pub data: Vec<u8>,
}

impl DataWriter {
    pub fn new() -> DataWriter {
        DataWriter { data: Vec::new() }
    }

    pub fn with_capacity(capacity: usize) -> DataWriter {
        DataWriter {
            data: Vec::with_capacity(capacity),
        }
    }

    pub fn write_string(&mut self, string: &String) {
        self.write_varint(string.len() as i32);
        self.data.extend_from_slice(string.as_bytes());
    }

    pub fn write_varint(&mut self, mut value: i32) {
        self.data
            .extend_from_slice(DataWriter::var_num(value as u64).as_slice());
    }

    pub fn write_varlong(&mut self, mut value: i64) {
        self.data
            .extend_from_slice(DataWriter::var_num(value as u64).as_slice());
    }

    pub fn write_u8(&mut self, value: u8) {
        self.data.push(value)
    }

    pub fn write_u16(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_u16_le(&mut self, value: u16) {
        self.data.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_i8(&mut self, value: i8) {
        self.data.push(value as u8)
    }

    pub fn write_i16(&mut self, value: i16) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_i32(&mut self, value: i32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_i64(&mut self, value: i64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_f32(&mut self, value: f32) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_f64(&mut self, value: f64) {
        self.data.extend_from_slice(&value.to_be_bytes());
    }

    pub fn write_bool(&mut self, value: bool) {
        self.data.push(if value { 0x01 } else { 0x00 })
    }

    pub fn write_vec_data(&mut self, data: &Vec<u8>) {
        self.data.append(&mut data.clone());
    }

    pub fn write_data(&mut self, data: &[u8]) {
        self.data.extend_from_slice(data);
    }

    pub fn write_position(&mut self, position: &Position) {
        self.data
            .extend_from_slice(&position.encode().to_be_bytes());
    }

    pub fn get_varint(value: u32) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend_from_slice(DataWriter::var_num(value as u64).as_slice());
        vec
    }

    pub fn var_num(mut value: u64) -> ArrayVec<u8, 10> {
        let mut array = ArrayVec::<u8, 10>::new();

        loop {
            let mut temp = (value & 0b01111111) as u8;
            value >>= 7;
            if value != 0 {
                temp |= 0b10000000;
                array.push(temp);
            } else {
                array.push(temp);
                break;
            }
        }

        array
    }
}
