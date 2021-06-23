pub struct Position {
    pub x: i32,
    pub y: i16,
    pub z: i32
}

impl Position {
    pub const fn encode(&self) -> i64 {
        ((self.x as i64 & 0x3FFFFFF) << 38) | ((self.y as i64 & 0xFFF) << 26) | (self.z as i64 & 0x3FFFFFF)
    }
}