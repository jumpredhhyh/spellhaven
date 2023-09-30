use bevy::log::info;
use noise::core::open_simplex::open_simplex_2d;
use noise::permutationtable::PermutationTable;
use noise::{Cache, Fbm, MultiFractal, NoiseFn, Perlin, Value, Worley};
use vox_format::VoxData;
use crate::chunk_generation::{BlockType, CHUNK_SIZE, LEVEL_OF_DETAIL};

pub fn generate_voxels(position: [i32; 3], tree_model: &Vec<Vec<Vec<BlockType>>>) -> ([[[BlockType; CHUNK_SIZE[2] + 2]; CHUNK_SIZE[1] + 2]; CHUNK_SIZE[0] + 2], i32, bool) {
    let mut blocks = [[[BlockType::Air; CHUNK_SIZE[2] + 2]; CHUNK_SIZE[1] + 2]; CHUNK_SIZE[0] + 2];
    let value_noise = Fbm::<Perlin>::new(2).set_frequency(0.5f64.powi(12) * LEVEL_OF_DETAIL as f64);
    let tree_noise = Cache::new(Worley::new(3));

    let hasher = PermutationTable::new(0);
    let roughness_hasher = PermutationTable::new(1);

    let (terrain_height, min_height) = generate_chunk_noise(&position);

    let mut generate_more: bool = false;

    for x in 0..CHUNK_SIZE[0] + 2 {
        for z in 0..CHUNK_SIZE[2] + 2 {
            let total_x = position[0] * CHUNK_SIZE[0] as i32 + x as i32;
            let total_z = position[2] * CHUNK_SIZE[2] as i32 + z as i32;

            let dryness = value_noise.get([total_x as f64, total_z as f64]);

            let noise_height = terrain_height[x][z];

            let tree_offset_x = total_x / 5;
            let tree_offset_z = total_z / 5;
            let tree_value = tree_noise.get([tree_offset_x as f64 * 12., tree_offset_z as f64 * 12.]);

            for y in min_height as usize..noise_height.min((CHUNK_SIZE[1] + 2 + min_height as usize) as f64) as usize {
                if y == CHUNK_SIZE[1] + 1 + min_height as usize { generate_more = true; }
                blocks[x][y - min_height as usize][z] = if y + 1 == noise_height.floor() as usize { if dryness < 0. { BlockType::Grass } else { BlockType::Sand } } else { BlockType::Stone }
                //blocks[x][y][z] = BlockType::Gray(((tree_value) * 255.) as u8)
            }

            if tree_value > 0. {
                let tree_x = (total_x - tree_offset_x * 5).abs() as usize;
                let tree_z = (total_z - tree_offset_z * 5).abs() as usize;

                let tree_noise_height_x = tree_offset_x * 5 + (5 / 2) * tree_offset_x.clamp(-1 , 1);
                let tree_noise_height_z = tree_offset_z * 5 + (5 / 2) * tree_offset_z.clamp(-1 , 1);

                let roughness = noise(tree_noise_height_x, tree_noise_height_z, 0.5f64.powi(9) * LEVEL_OF_DETAIL as f64, 0.2 / LEVEL_OF_DETAIL as f64, &roughness_hasher) - 0.15;
                let noise_height = fractal_noise(tree_noise_height_x, tree_noise_height_z, 0.5f64.powi(8) * LEVEL_OF_DETAIL as f64, 128. / LEVEL_OF_DETAIL as f64, 7, 2., 0.5 + roughness, &hasher);

                for (index, sub_tree) in tree_model[tree_x].iter().enumerate() {
                    if (noise_height as i32 - min_height + index as i32) < 0 {
                        continue;
                    }
                    if noise_height as usize + index - min_height as usize >= CHUNK_SIZE[1] + 2 {
                        if sub_tree[tree_z] == BlockType::Air { continue; }
                        generate_more = true;
                        break;
                    }
                    if sub_tree[tree_z] == BlockType::Air { continue; }
                    blocks[x][noise_height as usize + index - min_height as usize][z] = sub_tree[tree_z];
                }
            }
        }
    }

    (blocks, min_height, generate_more)
}

fn generate_chunk_noise(position: &[i32; 3]) -> ([[f64; CHUNK_SIZE[2] + 2]; CHUNK_SIZE[0] + 2], i32) {
    let mut result = [[0f64; CHUNK_SIZE[2] + 2]; CHUNK_SIZE[0] + 2];

    let mut min = f64::MAX;

    let hasher = PermutationTable::new(0);
    let roughness_hasher = PermutationTable::new(1);

    for x in 0..CHUNK_SIZE[0] + 2 {
        for z in 0..CHUNK_SIZE[2] + 2 {
            let total_x = position[0] * CHUNK_SIZE[0] as i32 + x as i32;
            let total_z = position[2] * CHUNK_SIZE[2] as i32 + z as i32;

            let roughness = noise(total_x, total_z, 0.5f64.powi(9) * LEVEL_OF_DETAIL as f64, 0.2 / LEVEL_OF_DETAIL as f64, &roughness_hasher) - 0.15;
            let noise_height = fractal_noise(total_x, total_z, 0.5f64.powi(8) * LEVEL_OF_DETAIL as f64, 128. / LEVEL_OF_DETAIL as f64, 7, 2., 0.5 + roughness, &hasher);
            min = min.min(noise_height);
            result[x][z] = noise_height;
        }
    }

    (result, min as i32 - 2 + position[1] * CHUNK_SIZE[1] as i32)
}

fn fractal_noise(x: i32, z: i32, frequency: f64, amplitude: f64, octaves: i32, lacunarity: f64, persistence: f64, hasher: &PermutationTable) -> f64 {
    let mut noise_value: f64 = 0.;

    for octave in 0..octaves {
        noise_value += noise(x, z, frequency * lacunarity.powi(octave), amplitude * persistence.powi(octave), hasher)
    }

    noise_value
}

fn noise(x: i32, z: i32, frequency: f64, amplitude: f64, hasher: &PermutationTable) -> f64 {
    (open_simplex_2d([x as f64 * frequency, z as f64 * frequency], hasher) + 0.5) * amplitude
}

pub fn vox_data_to_blocks(vox_data: VoxData) -> Vec<Vec<Vec<BlockType>>> {
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

const TREE: [[[BlockType; 5]; 5]; 5] = [[[BlockType::Custom(255, 0, 0); 5]; 5]; 5];