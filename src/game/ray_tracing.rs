use std::f64::consts::PI;

pub fn ray_casting(matrix: &mut [[i8; 16]; 16], source: &PosValue) {
    let mut angle = 0f64;
    while angle <= 2f64 * PI {
        for radius in 0..source.value {
            let posx = ((radius as f64 * angle.cos()).round()) as i8;
            let posz = ((radius as f64 * angle.sin()).round()) as i8;

            let posx = source.pos.x as i8 + posx;
            let posz = source.pos.z as i8 + posz;

            if matrix.len() <= posx as usize || matrix.len() <= posz as usize {
                continue;
            }
            let value = matrix[posx as usize][posz as usize];
            if value < 0 {
                angle += 0.45f64;
                break;
            }
            if value < source.value - radius && value != 0 {
                continue;
            }
            matrix[posx as usize][posz as usize] = source.value - radius;
        }
        angle += 0.01f64;
    }
}

pub fn print_matrix(matrix: &[[u8; 16]; 16]) {
    for x in matrix {
        print!("| ");
        for z in x {
            if *z == 0 {
                print!("▀ ");
            }
            // else if *z == -1 {
            //     print!(" ");
            // }
            else {
                print!("{} ", z)
            }
        }
        print!("|");
        println!();
    }
}

pub struct Pos {
    x: usize,
    z: usize
}

pub struct PosValue {
    pos: Pos,
    value: i8
}

impl Pos {
    pub fn set(&self, matrix: &mut [[i8; 16]; 16], value: i8) {
        matrix[self.x][self.z] = value;
    }
}

impl PosValue {
    pub fn set(&self, matrix: &mut [[i8; 16]; 16]) {
        matrix[self.pos.x][self.pos.z] = self.value;
    }

    pub const fn new(x: usize, z: usize, value: i8) -> PosValue {
        PosValue {pos: Pos {x, z}, value}
    }
}
