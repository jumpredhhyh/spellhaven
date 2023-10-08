use std::sync::Arc;
use bevy::prelude::Resource;
use noise::Abs;
use vox_format::VoxData;
use crate::chunk_generation::BlockType;
use crate::voxel_generation::StructureGenerator;
use crate::white_noise::WhiteNoise;

#[derive(Resource)]
pub struct GenerationOptionsResource(pub Arc<GenerationOptions>);

impl Default for GenerationOptionsResource {
    fn default() -> Self {
        let tree = vox_data_to_structure_data(&vox_format::from_file("assets/tree_2.vox").unwrap());
        let tree_house = vox_data_to_structure_data(&vox_format::from_file("assets/tree_house.vox").unwrap());

        Self {
            0: Arc::new(GenerationOptions {
                structures: vec![
                    StructureGenerator {
                        model: tree.0.clone(),
                        model_size: tree.1,
                        noise: Arc::new(Abs::new(WhiteNoise::new(1))),
                        generation_size: [30, 30],
                        grid_offset: [15, 15],
                        generate_debug_blocks: false
                    },
                    StructureGenerator {
                        model: tree.0.clone(),
                        model_size: tree.1,
                        noise: Arc::new(Abs::new(WhiteNoise::new(2))),
                        generation_size: [30, 30],
                        grid_offset: [0, 0],
                        generate_debug_blocks: false
                    },
                    StructureGenerator {
                        model: tree_house.0.clone(),
                        model_size: tree_house.1,
                        noise: Arc::new(Abs::new(WhiteNoise::new(3))),
                        generation_size: [1000, 1000],
                        grid_offset: [7, 11],
                        generate_debug_blocks: false
                    }
                ]
            }),
        }
    }
}

pub struct GenerationOptions {
    pub structures: Vec<StructureGenerator>
}


fn vox_data_to_blocks(vox_data: &VoxData) -> Vec<Vec<Vec<BlockType>>> {
    let model = vox_data.models.first().unwrap();
    let mut result: Vec<Vec<Vec<BlockType>>> = Vec::with_capacity(model.size.x as usize);
    for x in 0..model.size.x {
        result.push(Vec::with_capacity(model.size.z as usize));
        for y in 0..model.size.z {
            result[x as usize].push(Vec::with_capacity(model.size.y as usize));
            for _ in 0..model.size.y {
                result[x as usize][y as usize].push(BlockType::Air);
            }
        }
    }

    for voxel in model.voxels.iter() {
        let color = vox_data.palette.colors[voxel.color_index.0 as usize];
        result[voxel.point.x as usize][voxel.point.z as usize][voxel.point.y as usize] = BlockType::Custom(color.r, color.g, color.b);
    }

    result
}

fn vox_data_model_size(vox_data: &VoxData) -> [i32; 3] {
    let model_size = vox_data.models.first().unwrap().size;
    [model_size.x as i32, model_size.z as i32, model_size.y as i32]
}

fn vox_data_to_structure_data(vox_data: &VoxData) -> (Arc<Vec<Vec<Vec<BlockType>>>>, [i32; 3]) {
    (Arc::new(vox_data_to_blocks(vox_data)), vox_data_model_size(vox_data))
}