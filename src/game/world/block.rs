#[derive(Default, Copy)]
pub struct Block {
    pub material: Material,
    pub metadata: u8,
}

impl Block {
    pub fn new() -> Self {
        Block {
            material: Material::default(),
            metadata: 0,
        }
    }

    pub fn from_material(material: Material) -> Self {
        Block {
            material,
            metadata: 0,
        }
    }

    pub fn from_material_and_metadata(material: Material, metadata: u8) -> Self {
        Block { material, metadata }
    }

    pub fn from_encoded(data: u16) -> Self {
        Block {
            material: Material { id: data >> 4 },
            metadata: (data & 0xF) as u8,
        }
    }

    pub fn get_encoded(&self) -> u16 {
        self.material.id << 4 | self.metadata as u16
    }
}

impl Clone for Block {
    fn clone(&self) -> Self {
        Block {
            material: self.material,
            metadata: self.metadata,
        }
    }
}

#[derive(Default, Copy, Clone)]
pub struct Material {
    pub id: u16,
}
