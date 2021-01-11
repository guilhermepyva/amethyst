use crate::utils;

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
            return Err("data size is longer than datareader remaining bytes")
        }

        let mut data = vec![0; length];

        for x in self.cursor..length + self.cursor {
            data[x - self.cursor] = self.data[x];
        }

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
            if num_read > 5 {
                return Err("VarInt is too big")
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
        if !self.check_lenght(2) {
            return Err("data size is longer than datareader remaining bytes")
        }

        let u16 = ((self.data[self.cursor] as u16) << 8) + (self.data[self.cursor + 1] as u16);

        self.cursor += 2;
        return Ok(u16);
    }

    pub fn read_i64<'a>(&mut self) -> Result<i64, &'a str> {
        if !self.check_lenght(8) {
            return Err("data size is longer than datareader remaining bytes")
        }

        let mut result: i64 = 0;

        for x in 0..8 {
            result += self.data[self.cursor + x] as i64;
            if x != 7 {
                result <<= 8;
            }
        }

        self.cursor += 8;
        return Ok(result);
    }

    pub fn read_string<'a>(&mut self) -> Result<String, &'a str> {
        if !self.check_lenght(2) {
            return Err("data size is longer than datareader remaining bytes")
        }

        let string_lenght = self.data[self.cursor] as usize;
        if string_lenght == 0 {
            return Ok(String::new())
        }
        let vec = utils::arrays::extract_vector(self.data, self.cursor + 1, string_lenght + self.cursor + 1);

        self.cursor += string_lenght + 1;

        return match String::from_utf8(vec) {
            Ok(t) => Ok(t),
            Err(_e) => Err("couldn't convert bytes to string")
        }
    }

    fn check_lenght(&self, lenght: usize) -> bool {
        if lenght + self.cursor > self.data.len() {
            return false;
        }
        return true;
    }
}