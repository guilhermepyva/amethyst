use crate::game::world::block::Block;
use crate::game::world::chunk::{ChunkColumn, ChunkPos};
use crate::game::world::coords::Position;
use std::time::Instant;

pub struct World {
    pub name: String,
    pub difficulty: u8,
    pub level_type: LevelType,
    pub chunks: Vec<ChunkColumn>,
}

impl World {
    pub fn new(name: String, difficulty: u8, level_type: LevelType) -> Self {
        World {
            name,
            difficulty,
            level_type,
            chunks: Vec::new(),
        }
    }

    pub fn set_block(&mut self, block: Block, x: i32, y: i16, z: i32) {
        let chunk_pos = ChunkPos::from_block_coords(x, z);

        let chunk = self.allocate_chunk(chunk_pos);
        let section = chunk.allocate_section((y / 16) as usize);

        section.blocks[(y.abs() % 16) as usize][(z.rem_euclid(16)) as usize]
            [(x.rem_euclid(16)) as usize] = block.get_encoded();
    }

    pub fn get_block(&self, x: i32, y: i16, z: i32) -> Block {
        let chunk_pos = ChunkPos::from_block_coords(x, z);

        return match self.get_chunk(chunk_pos) {
            None => Block::default(),
            Some(t) => {
                let section_pos = y / 16;
                match &t.sections[section_pos as usize] {
                    None => Block::default(),
                    Some(section) => Block::from_encoded(
                        section.blocks[(y % 16) as usize][z as usize][x as usize],
                    ),
                }
            }
        };
    }

    pub fn set_block_light(&mut self, mut power: u8, x: i32, y: i32, z: i32) {
        let chunk_pos = ChunkPos::from_block_coords(x, z);

        let chunk = self.allocate_chunk(chunk_pos);
        let section = chunk.allocate_section((y / 16) as usize);

        let x = (x.rem_euclid(16)) as usize;
        let y = (y.abs() % 16) as usize;
        let z = (z.rem_euclid(16)) as usize;

        let even = x % 2 != 0;
        let index = y * 128 + z * 8 + (x / 2);

        if even {
            power <<= 4;
        }
        section.block_light[index] &= if even { 0x0F } else { 0xF0 };
        section.block_light[index] |= power;
    }

    pub fn set_light(&mut self, mut power: u8, x: i32, y: i32, z: i32) -> bool {
        let chunk_pos = ChunkPos::from_block_coords(x, z);

        let chunk = self.allocate_chunk(chunk_pos);
        let section = chunk.allocate_section((y / 16) as usize);

        let x = (x.rem_euclid(16)) as usize;
        let y = (y.abs() % 16) as usize;
        let z = (z.rem_euclid(16)) as usize;

        let odd = x % 2 != 0;
        let index = y * 128 + z * 8 + (x / 2);

        let current_light = section.block_light[index] >> (4 * (!odd as u8));

        if power < current_light {
            return false;
        }

        if odd {
            power <<= 4;
        }

        section.block_light[index] &= if odd { 0x0F } else { 0xF0 };
        section.block_light[index] |= power;
        true
    }

    pub fn flood_fill_light(&mut self, mut power: u8, x: i32, y: i32, z: i32) {
        if !self.set_light(power, x, y, z) {
            return;
        }

        if power == 1 {
            return;
        }

        let power = power - 1;
        self.flood_fill_light(power, y + 1, z, x);
        self.flood_fill_light(power, y - 1, z, x);
        self.flood_fill_light(power, y, z + 1, x);
        self.flood_fill_light(power, y, z - 1, x);
        self.flood_fill_light(power, y, z, x + 1);
        self.flood_fill_light(power, y, z, x - 1);
    }

    pub fn get_chunk(&self, chunk_pos: ChunkPos) -> Option<&ChunkColumn> {
        self.chunks
            .iter()
            .find(|x| x.get_chunk_pos().eq(&chunk_pos))
    }

    pub fn allocate_chunk(&mut self, chunk_pos: ChunkPos) -> &mut ChunkColumn {
        let index = self
            .chunks
            .iter()
            .position(|x| x.get_chunk_pos().eq(&chunk_pos));
        match index {
            None => {
                self.chunks.push(ChunkColumn::new(chunk_pos));
                self.chunks.last_mut().unwrap()
            }
            Some(x) => &mut self.chunks[x],
        }
    }
}

pub enum LevelType {
    Default,
    Flat,
    LargeBiomes,
    Amplified,
    Default11,
}

impl LevelType {
    pub fn to_str(&self) -> &str {
        match self {
            LevelType::Default => "default",
            LevelType::Flat => "flat",
            LevelType::LargeBiomes => "largeBiomes",
            LevelType::Amplified => "amplified",
            LevelType::Default11 => "default_1_1",
        }
    }
}
