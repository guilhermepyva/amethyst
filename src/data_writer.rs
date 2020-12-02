pub struct DataWriter {
    pub data: Vec<u8>
}

impl DataWriter {
    pub fn new() -> DataWriter {
        DataWriter { data: Vec::new() }
    }

    pub fn write_string(&mut self, string: &String) {
        self.write_varint(string.len() as u32);
        self.data.append(&mut string.as_bytes().to_vec());
    }

    pub fn write_varint(&mut self, mut value: u32) {
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

    pub fn write_i64(&mut self, mut value: i64) {
        for x in 0..8 {
            let temp = (value & 0b00000000) as u8;
            value >>= 8;
            self.data.push(temp);
        }
    }

    pub fn write_data(&mut self, data: &Vec<u8>) {
        self.data.append(&mut data.clone());
    }

    pub fn set_lenght(&mut self, lenght: u32) {
        let varint = get_varint(lenght);
        self.data.splice(0..0, varint);
    }
}

fn get_varint(mut value: u32) -> Vec<u8> {
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