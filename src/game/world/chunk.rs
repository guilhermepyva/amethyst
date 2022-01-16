use crate::data_writer::DataWriter;
use crate::game::packets::{ExtendedPacket, Packet};
use crate::game::world::block::{Block, Material};
use crate::game::world::world::LevelType::Default11;
use std::mem::size_of_val;
use std::str::from_boxed_utf8_unchecked;
use std::time::{Duration, Instant};
use regex::internal::Inst;

#[derive(Copy, Clone, Default, Eq, PartialEq, Hash)]
pub struct ChunkPos {
    pub x: i32,
    pub z: i32,
}

impl ChunkPos {
    pub fn from_block_coords(x: i32, z: i32) -> Self {
        ChunkPos {
            x: x.div_euclid(16),
            z: z.div_euclid(16),
        }
    }
}

pub struct ChunkSection {
    pub blocks: [[[u16; 16]; 16]; 16],
    pub block_light: [u8; 2048],
    pub sky_light: [u8; 2048],
}

pub struct ChunkColumn {
    chunk_pos: ChunkPos,
    pub sections: [Option<Box<ChunkSection>>; 16],
}

impl ChunkSection {
    pub fn new() -> Self {
        ChunkSection {
            blocks: [[[0; 16]; 16]; 16],
            block_light: [0; 2048],
            sky_light: [0; 2048],
        }
    }

    pub const CHUNK_SECTION_PACKET_SIZE: usize = 12288;
    pub const CHUNK_BIOME_SIZE: usize = 256;
}

impl Default for ChunkSection {
    fn default() -> Self {
        ChunkSection::new()
    }
}

impl ChunkColumn {
    pub fn new(chunk_pos: ChunkPos) -> Self {
        ChunkColumn {
            chunk_pos,
            sections: Default::default(),
        }
    }

    pub fn allocate_section(&mut self, section: usize) -> &mut Box<ChunkSection> {
        if self.sections[section].is_some() {
            return self.sections[section].as_mut().unwrap();
        }
        self.sections[section] = Some(Box::new(ChunkSection::new()));
        self.sections[section].as_mut().unwrap()
    }

    pub fn set_block(&mut self, block: Block, y: u8, z: u8, x: u8) {
        let section = self.allocate_section((y / 16) as usize);

        section.blocks[(y % 16) as usize][z as usize][x as usize] = block.get_encoded();
    }

    pub fn get_block(&self, y: u8, z: u8, x: u8) -> Block {
        let chunk_pos = y / 16;
        return match &self.sections[chunk_pos as usize] {
            None => Block::default(),
            Some(section) => {
                Block::from_encoded(section.blocks[(y % 16) as usize][z as usize][x as usize])
            }
        };
    }

    pub fn bitmask(&self) -> (u16, usize) {
        let mut alive_sections = 0usize;
        let mut bitmask = 0u16;
        for y in self.sections.iter().enumerate() {
            if y.1.is_some() {
                bitmask |= 1 << y.0;
                alive_sections += 1;
            }
        }
        (bitmask, alive_sections)
    }

    pub fn write_chunk_data(&self) -> ExtendedPacket {
        let bitmask = self.bitmask();

        let mut vec = Vec::with_capacity((bitmask.1 * ChunkSection::CHUNK_SECTION_PACKET_SIZE) + ChunkSection::CHUNK_BIOME_SIZE);
        self.write(&mut vec);

        ExtendedPacket::ChunkData {
            x: self.chunk_pos.x,
            y: self.chunk_pos.z,
            ground_up_continuous: true,
            bitmask: bitmask.0,
            data: vec
        }
    }

    pub fn write(&self, data: &mut Vec<u8>) {
        let iterator = self.sections.iter().filter_map(|x| x.as_ref());

        iterator.clone().for_each(|x| data.extend_from_slice(&bytemuck::cast_slice(&x.blocks)));
        iterator.clone().for_each(|x| data.extend_from_slice(&x.block_light));
        iterator.clone().for_each(|x| data.extend_from_slice(&x.sky_light));

        data.extend_from_slice(&[0u8; 256]);
    }

    #[inline]
    pub fn get_chunk_pos(&self) -> &ChunkPos {
        &self.chunk_pos
    }
}
