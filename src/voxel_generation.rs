use std::sync::Arc;
use noise::{Cache, Fbm, MultiFractal, NoiseFn, Perlin};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use crate::chunk_generation::{BlockType, CHUNK_SIZE, LEVEL_OF_DETAIL};
use crate::fractal_open_simplex::FractalOpenSimplex;
use crate::generation_options::GenerationOptions;
use crate::roughness::Roughness;

pub struct StructureGenerator {
    pub model: Arc<Vec<Vec<Vec<BlockType>>>>,
    pub model_size: [i32; 3],
    pub noise: Arc<dyn NoiseFn<f64, 2> + Send + Sync>,
    pub generation_size: [i32; 2],
    pub grid_offset: [i32; 2],
    pub generate_debug_blocks: bool,
    //pub height_offset: i32
}

pub fn generate_voxels(position: [i32; 3], generation_options: &GenerationOptions) -> ([[[BlockType; CHUNK_SIZE[2] + 2]; CHUNK_SIZE[1] + 2]; CHUNK_SIZE[0] + 2], i32, bool) {
    let mut blocks = [[[BlockType::Air; CHUNK_SIZE[2] + 2]; CHUNK_SIZE[1] + 2]; CHUNK_SIZE[0] + 2];
    let value_noise = Fbm::<Perlin>::new(2).set_frequency(0.5f64.powi(12) * LEVEL_OF_DETAIL as f64);

    let noise = FractalOpenSimplex::new(
        0,
        0.5f64.powi(9) * LEVEL_OF_DETAIL as f64,
        256. / LEVEL_OF_DETAIL as f64,
        7,
        2.,
        0.5,
        Roughness::new(1, 0.5f64.powi(10) * LEVEL_OF_DETAIL as f64, 0.2 / LEVEL_OF_DETAIL as f64)
    );

    let (terrain_height, min_height) = generate_chunk_noise(&position, &noise);

    let mut generate_more: bool = false;

    for x in 0..CHUNK_SIZE[0] + 2 {
        for z in 0..CHUNK_SIZE[2] + 2 {
            let total_x = position[0] * CHUNK_SIZE[0] as i32 + x as i32;
            let total_z = position[2] * CHUNK_SIZE[2] as i32 + z as i32;

            let dryness = value_noise.get([total_x as f64, total_z as f64]);

            let noise_height = terrain_height[x][z];

            for y in min_height as usize..noise_height.min((CHUNK_SIZE[1] + 2 + min_height as usize) as f64) as usize {
                if y == CHUNK_SIZE[1] + 1 + min_height as usize { generate_more = true; }
                blocks[x][y - min_height as usize][z] = if y + 1 == noise_height.floor() as usize { if dryness < 0. { BlockType::Grass } else { BlockType::Sand } } else { BlockType::Stone }
            }

            for structure in &generation_options.structures {
                let structure_offset_x = (total_x + structure.grid_offset[0]).div_floor(structure.generation_size[0]);
                let structure_offset_z = (total_z + structure.grid_offset[1]).div_floor(structure.generation_size[1]);
                let structure_value = structure.noise.get([structure_offset_x as f64, structure_offset_z as f64]);
                if structure.generate_debug_blocks {
                    blocks[x][(noise_height.min(CHUNK_SIZE[1] as f64 + min_height as f64) as i32 - min_height.min(noise_height as i32)).max(1) as usize - 1][z] = BlockType::Gray(((structure_value) * 255.) as u8);
                }
                let mut rand = StdRng::seed_from_u64((structure_value.abs() * 10000.) as u64);

                if structure_value > 0. {
                    let random_x = rand.gen_range(0..=structure.generation_size[0] - structure.model_size[0]);
                    let random_z = rand.gen_range(0..=structure.generation_size[1] - structure.model_size[2]);

                    let structure_x = (total_x + structure.grid_offset[0] - structure_offset_x * structure.generation_size[0]).abs() - random_x;
                    let structure_z = (total_z + structure.grid_offset[1] - structure_offset_z * structure.generation_size[1]).abs() - random_z;

                    if structure_x < 0 || structure_z < 0 || structure_x >= structure.model_size[0] || structure_z >= structure.model_size[2] {
                        continue;
                    }

                    let structure_noise_height_x = structure_offset_x * structure.generation_size[0] + (structure.model_size[0] / 2) - structure.grid_offset[0] + random_x;
                    let structure_noise_height_z = structure_offset_z * structure.generation_size[1] + (structure.model_size[2] / 2) - structure.grid_offset[1] + random_z;

                    let noise_height = noise.get([structure_noise_height_x as f64, structure_noise_height_z as f64]);

                    for (index, sub_structure) in structure.model[structure_x as usize].iter().enumerate() {
                        if (noise_height as i32 - min_height + index as i32) < 0 {
                            continue;
                        }
                        let structure_block = sub_structure[structure_z as usize];
                        if structure_block == BlockType::Air { continue; }
                        if noise_height as usize + index - min_height as usize >= CHUNK_SIZE[1] + 2 {
                            generate_more = true;
                            break;
                        }
                        blocks[x][noise_height as usize + index - min_height as usize][z] = structure_block;
                    }
                }
            }
        }
    }

    (blocks, min_height, generate_more)
}

fn generate_chunk_noise<N>(position: &[i32; 3], noise: &N) -> ([[f64; CHUNK_SIZE[2] + 2]; CHUNK_SIZE[0] + 2], i32) where N: NoiseFn<f64, 2usize> {
    let mut result = [[0f64; CHUNK_SIZE[2] + 2]; CHUNK_SIZE[0] + 2];

    let mut min = f64::MAX;

    for x in 0..CHUNK_SIZE[0] + 2 {
        for z in 0..CHUNK_SIZE[2] + 2 {
            let total_x = position[0] * CHUNK_SIZE[0] as i32 + x as i32;
            let total_z = position[2] * CHUNK_SIZE[2] as i32 + z as i32;

            let noise_height = noise.get([total_x as f64, total_z as f64]);
            min = min.min(noise_height);
            result[x][z] = noise_height;
        }
    }

    (result, min as i32 - 2 + position[1] * CHUNK_SIZE[1] as i32)
}