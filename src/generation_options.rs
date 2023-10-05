use std::sync::Arc;
use noise::{Abs, Cache};
use crate::chunk_generation::BlockType;
use crate::voxel_generation::StructureGenerator;
use crate::white_noise::WhiteNoise;

pub struct GenerationOptions {
    pub structures: Vec<StructureGenerator>
}

pub struct GenerationAssets {
    pub tree: (Arc<Vec<Vec<Vec<BlockType>>>>, [i32; 3]),
    pub tree_house: (Arc<Vec<Vec<Vec<BlockType>>>>, [i32; 3])
}

impl GenerationOptions {
    pub fn get_options(generation_assets: Arc<GenerationAssets>) -> GenerationOptions {
        GenerationOptions {
            structures: vec![
                StructureGenerator {
                    model: generation_assets.tree.0.clone(),
                    model_size: generation_assets.tree.1,
                    noise: Box::new(Cache::new(Abs::new(WhiteNoise::new(1)))),
                    generation_size: [15, 15],
                    grid_offset: [3, 5],
                    generate_debug_blocks: false
                },
                StructureGenerator {
                    model: generation_assets.tree.0.clone(),
                    model_size: generation_assets.tree.1,
                    noise: Box::new(Cache::new(Abs::new(WhiteNoise::new(2)))),
                    generation_size: [15, 15],
                    grid_offset: [0, 0],
                    generate_debug_blocks: false
                },
                StructureGenerator {
                    model: generation_assets.tree_house.0.clone(),
                    model_size: generation_assets.tree_house.1,
                    noise: Box::new(Cache::new(Abs::new(WhiteNoise::new(3)))),
                    generation_size: [1000, 1000],
                    grid_offset: [7, 11],
                    generate_debug_blocks: false
                }
            ]
        }
    }
}