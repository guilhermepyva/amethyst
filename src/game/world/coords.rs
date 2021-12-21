#[derive(Clone, Eq, PartialEq, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i16,
    pub z: i32,
}

impl Position {
    pub const fn encode(&self) -> i64 {
        ((self.x as i64 & 0x3FFFFFF) << 38)
            | ((self.y as i64 & 0xFFF) << 26)
            | (self.z as i64 & 0x3FFFFFF)
    }
}

#[derive(Clone, PartialEq)]
pub struct Point {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Point {
    #[inline]
    pub fn absolute_x(&self) -> i32 {
        (self.x * 32f64) as i32
    }

    #[inline]
    pub fn absolute_y(&self) -> i32 {
        (self.y * 32f64) as i32
    }

    #[inline]
    pub fn absolute_z(&self) -> i32 {
        (self.z * 32f64) as i32
    }

    #[inline]
    pub fn absolute(&self) -> Position {
        Position {
            x: self.absolute_x(),
            y: self.absolute_y() as i16,
            z: self.absolute_z(),
        }
    }
}
