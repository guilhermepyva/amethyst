use std::convert::{TryFrom, TryInto};

pub struct DataReader<'a> {
    pub data: &'a Vec<u8>,
    pub cursor: usize
}

impl DataReader<'_> {
    pub fn new(data: &Vec<u8>) -> DataReader {
        DataReader { data, cursor: 0 }
    }
    pub fn new_on_cursor(data: &Vec<u8>, cursor: usize) -> DataReader {
        DataReader { data, cursor }
    }

    pub fn read_data_fixed<'a>(&mut self, length: usize) -> Result<Vec<u8>, &'a str> {
        if !self.check_lenght(length) {
            return Err("data size is longer than data reader remaining bytes")
        }

        let mut data = self.data[self.cursor..self.cursor + length].to_vec();

        self.cursor += length;
        Ok(data)
    }

    pub fn read_data(&mut self) -> Result<Vec<u8>, &str> {
        let length = match self.read_varint() {
            Ok(t) => t,
            Err(e) => return Err(e)
        };

        self.read_data_fixed(length as usize)
    }

    pub fn read_varint<'a>(&mut self) -> Result<i32, &'a str> {
        let mut result: i32 = 0;
        let mut num_read: u8 = 0;
        let mut read: u8;

        loop {
            read = match self.read_u8() {
                Err(_) => return Ok(result),
                Ok(t) => t
            };
            result += (((read as i8) & 0b01111111) as i32) << (7 * num_read);

            num_read += 1;
            if num_read > 5 {
                return Err("VarInt is too big")
            }
            if (read & 0b10000000) == 0 {
                return Ok(result)
            }
        }
    }

    pub fn read_varlong<'a>(&mut self) -> Result<u128, &'a str> {
        let mut result: u128 = 0;
        let mut num_read: u8 = 0;
        let mut read: u8;

        loop {
            read = match self.read_u8() {
                Err(_) => return Ok(result),
                Ok(t) => t
            };
            result += ((read & 0b01111111) as u128) << (7 * num_read);

            num_read += 1;
            if num_read > 10 {
                return Err("VarLong is too big")
            }
            if (read & 0b10000000) == 0 {
                return Ok(result)
            }
        }
    }

    pub fn read_u8<'a>(&mut self) -> Result<u8, &'a str> {
        if !self.check_lenght(1) {
            return Err("data size is longer than datareader remaining bytes")
        }

        self.cursor += 1;

        return Ok(self.data[self.cursor - 1]);
    }

    pub fn read_u16<'a>(&mut self) -> Result<u16, &'a str> {
        let n = u16::from_be_bytes(match self.data[self.cursor..self.cursor + 2].try_into() {
            Ok(t) => t,
            Err(_e) => return Err("data size is longer than datareader remaining bytes")
        });

        self.cursor += 2;
        return Ok(n);
    }

    pub fn read_i64<'a>(&mut self) -> Result<i64, &'a str> {
        let n = i64::from_be_bytes(match self.data[self.cursor..self.cursor + 8].try_into() {
            Ok(t) => t,
            Err(_e) => return Err("data size is longer than datareader remaining bytes")
        });

        self.cursor += 8;
        return Ok(n);
    }

    pub fn read_string<'a>(&mut self) -> Result<String, &'a str> {
        let string_length = self.read_varint()? as usize;

        if string_length == 0 {
            return Ok(String::new());
        }

        if !self.check_lenght(string_length) {
            return Err("data size is longer than datareader remaining bytes")
        }

        let vec = self.data[self.cursor..string_length + self.cursor].to_vec();

        self.cursor += string_length;

        return match String::from_utf8(vec) {
            Ok(t) => Ok(t),
            Err(_e) => Err("couldn't convert bytes to string")
        }
    }

    #[inline]
    fn check_lenght(&self, lenght: usize) -> bool {
        if lenght + self.cursor > self.data.len() {
            return false;
        }
        return true;
    }
}