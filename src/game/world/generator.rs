use crate::game::world::block::{Block, Material};
use crate::game::world::chunk::ChunkColumn;
use crate::game::world::world::World;
use std::time::Instant;

pub fn generate(world: &mut World) {
    let stone = Block::from_material(Material { id: 2 });
    let torch = Block::from_material_and_metadata(Material { id: 50 }, 5);

    for x in -15..15 {
        for z in -15..15 {
            world.set_block(stone, x, 49, z);
        }
    }

    world.set_block(torch, 0, 50, 0);
    world.set_block(stone, -1, 50, 0);
    let now = Instant::now();
    world.flood_fill_light(15, 0, 50, 0);
    println!("{:?}", now.elapsed());
}
