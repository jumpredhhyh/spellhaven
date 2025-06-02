use crate::world_generation::{
    chunk_generation::{BlockType, VOXEL_SIZE},
    foliage_generation::tree_l_system::TreeLSystem,
    voxel_world::ChunkLod,
};
use bevy::{math::IVec2, utils::HashMap};
use fastnoise_lite::FastNoiseLite;
use rand::{rngs::StdRng, SeedableRng};
use std::{cell::RefCell, rc::Rc, sync::Arc};

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
    fn get_structure_model(
        &self,
        structure_position: IVec2,
        lod: ChunkLod,
    ) -> Rc<Vec<Vec<Vec<BlockType>>>>;
}

pub struct FixedStructureGenerator {
    pub fixed_structure_metadata: VoxelStructureMetadata,
    pub fixed_structure_model: Arc<Vec<Vec<Vec<BlockType>>>>,
}

impl StructureGenerator for FixedStructureGenerator {
    fn get_structure_metadata(&self) -> &VoxelStructureMetadata {
        &self.fixed_structure_metadata
    }

    fn get_structure_model(&self, _: IVec2, _: ChunkLod) -> Rc<Vec<Vec<Vec<BlockType>>>> {
        Rc::new(self.fixed_structure_model.to_vec())
    }
}

pub struct TreeStructureGenerator {
    pub fixed_structure_metadata: VoxelStructureMetadata,
}

const TREE_VOXEL_SIZE: usize = (27f32 / VOXEL_SIZE) as usize;

impl TreeStructureGenerator {
    pub fn new(metadata: VoxelStructureMetadata) -> Self {
        let model_size = [
            (metadata.model_size[0] as f32 / VOXEL_SIZE) as i32,
            (metadata.model_size[1] as f32 / VOXEL_SIZE) as i32,
            (metadata.model_size[2] as f32 / VOXEL_SIZE) as i32,
        ];

        let generation_size = [
            (metadata.generation_size[0] as f32 / VOXEL_SIZE) as i32,
            (metadata.generation_size[1] as f32 / VOXEL_SIZE) as i32,
        ];

        let grid_offset = [
            (metadata.grid_offset[0] as f32 / VOXEL_SIZE) as i32,
            (metadata.grid_offset[1] as f32 / VOXEL_SIZE) as i32,
        ];

        Self {
            fixed_structure_metadata: VoxelStructureMetadata {
                model_size,
                generation_size,
                grid_offset,
                generate_debug_blocks: metadata.generate_debug_blocks,
                debug_rgb_multiplier: metadata.debug_rgb_multiplier,
                noise: metadata.noise,
            },
        }
    }
}

impl StructureGenerator for TreeStructureGenerator {
    fn get_structure_metadata(&self) -> &VoxelStructureMetadata {
        return &self.fixed_structure_metadata;
    }

    fn get_structure_model(
        &self,
        structure_position: IVec2,
        _: ChunkLod,
    ) -> Rc<Vec<Vec<Vec<BlockType>>>> {
        let noise_value = self
            .get_structure_metadata()
            .noise
            .get_noise_2d(structure_position.x as f32, structure_position.y as f32)
            * 0.5
            + 0.5;
        let mut rng = StdRng::seed_from_u64((noise_value.abs() * 10000.) as u64);
        let voxel_grid =
            TreeLSystem::grow_new::<TREE_VOXEL_SIZE, TREE_VOXEL_SIZE, TREE_VOXEL_SIZE>(&mut rng);

        Rc::new(voxel_grid)
    }
}

pub struct StructureGeneratorCache {
    cache: RefCell<HashMap<IVec2, Rc<Vec<Vec<Vec<BlockType>>>>>>,
    structure_generator: Arc<Box<dyn StructureGenerator + Send + Sync>>,
}

impl StructureGeneratorCache {
    pub fn new(structure_generator: &Arc<Box<dyn StructureGenerator + Send + Sync>>) -> Self {
        Self {
            structure_generator: structure_generator.clone(),
            cache: RefCell::new(HashMap::new()),
        }
    }
}

impl StructureGenerator for StructureGeneratorCache {
    fn get_structure_metadata(&self) -> &VoxelStructureMetadata {
        self.structure_generator.get_structure_metadata()
    }

    fn get_structure_model(
        &self,
        structure_position: IVec2,
        lod: ChunkLod,
    ) -> Rc<Vec<Vec<Vec<BlockType>>>> {
        let structure_position = if lod.usize() >= ChunkLod::Eighth.usize() {
            IVec2::new(0, 0)
        } else {
            structure_position
        };

        let mut cache = self.cache.borrow_mut();
        if let Some(model) = cache.get(&structure_position) {
            return model.clone();
        }

        let model = self
            .structure_generator
            .get_structure_model(structure_position, lod);

        cache.insert(structure_position, model.clone());

        model
    }
}
