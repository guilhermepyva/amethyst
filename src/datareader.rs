use crate::utils;

pub struct DataReader<'a> {
    pub data: &'a Vec<u8>,
    cursor: usize
}

impl DataReader<'_> {
    pub fn new(data: &Vec<u8>) -> DataReader {
        DataReader { data, cursor: 0 }
    }

    pub fn read_varint(&mut self) -> Result<u64, &str> {
        let mut result: u64 = 0;
        let mut num_read: u8 = 0;
        let mut read: u8;

        loop {
            read = match self.read_u8() {
                Err(_) => return Ok(result),
                Ok(t) => t
            };
            result += ((read & 0b01111111) as u64) << (7 * num_read);

            num_read += 1;
            if num_read > 5 {
                return Err("VarInt is too big")
            }
            if (read & 0b10000000) == 0 {
                return Ok(result)
            }
        }
    }

    pub fn read_varlong(&mut self) -> Result<u128, &str> {
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

    pub fn read_u8(&mut self) -> Result<u8, &str> {
        if !self.check_lenght(1) {
            return Err("data size is longer than datareader remaining bytes")
        }

        self.cursor += 1;

        return Ok(self.data[self.cursor - 1]);
    }

    pub fn read_u16(&mut self) -> Result<u16, &str> {
        if !self.check_lenght(2) {
            return Err("data size is longer than datareader remaining bytes")
        }

        let u16 = ((self.data[self.cursor] as u16) << 8) + (self.data[self.cursor + 1] as u16);

        self.cursor += 2;
        return Ok(u16);
    }

    pub fn read_string(&mut self) -> Result<String, &str> {
        if !self.check_lenght(2) {
            return Err("data size is longer than datareader remaining bytes")
        }

        let string_lenght = self.data[self.cursor] as usize;
        let vec = utils::arrays::array_copy(self.data, self.cursor + 1, string_lenght + self.cursor + 1).unwrap();

        self.cursor += string_lenght + 1;

        return Ok(String::from_utf8(vec).unwrap());
    }

    fn check_lenght(&self, lenght: usize) -> bool {
        if lenght + self.cursor > self.data.len() {
            return false;
        }
        return true;
    }
}