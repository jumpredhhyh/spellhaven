use crate::world_generation::chunk_generation::BlockType;
use bevy::math::IVec2;
use fastnoise_lite::FastNoiseLite;
use std::sync::Arc;

pub struct VoxelStructureMetadata {
    pub model_size: [i32; 3],
    pub generation_size: [i32; 2],
    pub grid_offset: [i32; 2],
    pub generate_debug_blocks: bool,
    pub debug_rgb_multiplier: [f32; 3],
    pub noise: FastNoiseLite,
}

pub trait StructureGenerator {
    fn get_structure_metadata(&self) -> &VoxelStructureMetadata;
    fn get_structure_model(&self, structure_position: IVec2) -> &Vec<Vec<Vec<BlockType>>>;
}

pub struct FixedStructureGenerator {
    pub fixed_structure_metadata: VoxelStructureMetadata,
    pub fixed_structure_model: Arc<Vec<Vec<Vec<BlockType>>>>,
}

impl StructureGenerator for FixedStructureGenerator {
    fn get_structure_metadata(&self) -> &VoxelStructureMetadata {
        &self.fixed_structure_metadata
    }

    fn get_structure_model(&self, _: IVec2) -> &Vec<Vec<Vec<BlockType>>> {
        &self.fixed_structure_model
    }
}

pub struct TreeStructureGenerator {}
