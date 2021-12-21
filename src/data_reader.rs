use std::convert::{TryFrom, TryInto};

pub struct DataReader<'a> {
    pub data: &'a [u8],
    pub cursor: usize,
}

impl DataReader<'_> {
    pub fn new(data: &[u8]) -> DataReader {
        DataReader { data, cursor: 0 }
    }
    pub fn new_on_cursor(data: &[u8], cursor: usize) -> DataReader {
        DataReader { data, cursor }
    }

    pub fn read_data_fixed<'a>(&mut self, length: usize) -> Option<Vec<u8>> {
        if !self.check_lenght(length) {
            return None;
        }

        let mut data = &self.data[self.cursor..self.cursor + length];

        self.cursor += length;
        Some(data.to_vec())
    }

    pub fn read_data(&mut self) -> Option<Vec<u8>> {
        let length = self.read_varint()?;

        self.read_data_fixed(length as usize)
    }

    pub fn read_varint<'a>(&mut self) -> Option<i32> {
        let mut result: i32 = 0;
        let mut num_read: u8 = 0;
        let mut read: u8;

        loop {
            read = self.read_u8()?;
            result += (((read as i8) & 0b01111111) as i32) << (7 * num_read);

            num_read += 1;
            if num_read > 5 {
                return None;
            }
            if (read & 0b10000000) == 0 {
                return Some(result);
            }
        }
    }

    pub fn read_varlong<'a>(&mut self) -> Option<i64> {
        let mut result: i64 = 0;
        let mut num_read: u8 = 0;
        let mut read: u8;

        loop {
            read = self.read_u8()?;
            result += ((read & 0b01111111) as i64) << (7 * num_read);

            num_read += 1;
            if num_read > 10 {
                return None;
            }
            if (read & 0b10000000) == 0 {
                return Some(result);
            }
        }
    }

    #[inline]
    pub fn read_u8<'a>(&mut self) -> Option<u8> {
        if !self.check_lenght(1) {
            return None;
        }

        self.cursor += 1;

        return Some(self.data[self.cursor - 1]);
    }

    pub fn read_u16<'a>(&mut self) -> Option<u16> {
        if !self.check_lenght(2) {
            return None;
        }

        let n = u16::from_be_bytes(match self.data[self.cursor..self.cursor + 2].try_into() {
            Ok(t) => t,
            Err(_e) => return None,
        });

        self.cursor += 2;
        return Some(n);
    }

    pub fn read_i64<'a>(&mut self) -> Option<i64> {
        if !self.check_lenght(8) {
            return None;
        }

        let n = i64::from_be_bytes(match self.data[self.cursor..self.cursor + 8].try_into() {
            Ok(t) => t,
            Err(_e) => return None,
        });

        self.cursor += 8;
        return Some(n);
    }

    pub fn read_string<'a>(&mut self) -> Option<String> {
        let string_length = self.read_varint()? as usize;

        if string_length == 0 {
            return Some(String::new());
        }

        if !self.check_lenght(string_length) {
            return None;
        }

        let vec = self.data[self.cursor..string_length + self.cursor].to_vec();

        self.cursor += string_length;

        return match String::from_utf8(vec) {
            Ok(t) => Some(t),
            Err(_e) => None,
        };
    }

    #[inline]
    fn check_lenght(&self, lenght: usize) -> bool {
        if lenght + self.cursor > self.data.len() {
            return false;
        }
        return true;
    }
}
