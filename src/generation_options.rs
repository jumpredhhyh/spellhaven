use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::{Arc, RwLock};
use bevy::log::info;
use bevy::prelude::{IVec2, Resource};
use bracket_noise::prelude::FastNoise;
use bracket_noise::prelude::NoiseType::WhiteNoise;
use noise::NoiseFn;
use rand::Rng;
use vox_format::{from_file, VoxData};
use crate::chunk_generation::BlockType;
use crate::chunk_generation::BlockType::Custom;
use crate::voxel_generation::{get_terrain_noise, StructureGenerator};
use crate::voxel_world::ChunkLod;

#[derive(Resource)]
pub struct GenerationOptionsResource(pub Arc<GenerationOptions>);

impl Default for GenerationOptionsResource {
    fn default() -> Self {
        let tree = vox_data_to_structure_data(&from_file("assets/tree_2.vox").unwrap());
        let tree_house = vox_data_to_structure_data(&from_file("assets/tree_house.vox").unwrap());
        let box_structure = vox_data_to_structure_data(&from_file("assets/box.vox").unwrap());

        Self {
            0: Arc::new(GenerationOptions {
                country_cache: RwLock::new(HashMap::new()),
                structures: vec![
                    StructureGenerator {
                        model: tree.0.clone(),
                        model_size: tree.1,
                        noise: get_seeded_white_noise(1),
                        generation_size: [30, 30],
                        grid_offset: [15, 15],
                        generate_debug_blocks: false,
                        debug_rgb_multiplier: [1., 0., 0.],
                    },
                    StructureGenerator {
                        model: tree.0.clone(),
                        model_size: tree.1,
                        noise: get_seeded_white_noise(2),
                        generation_size: [30, 30],
                        grid_offset: [0, 0],
                        generate_debug_blocks: false,
                        debug_rgb_multiplier: [0., 1., 0.],
                    },
                    StructureGenerator {
                        model: tree_house.0.clone(),
                        model_size: tree_house.1,
                        noise: get_seeded_white_noise(3),
                        generation_size: [1000, 1000],
                        grid_offset: [7, 11],
                        generate_debug_blocks: false,
                        debug_rgb_multiplier: [1., 1., 1.],
                    }
                ],
                structure_assets: vec![StructureAsset((*box_structure.0).clone())],
            }),
        }
    }
}

fn get_seeded_white_noise(seed: u64) -> FastNoise {
    let mut noise = FastNoise::seeded(seed);
    noise.set_noise_type(WhiteNoise);
    noise.set_frequency(0.1);
    noise
}

pub struct GenerationOptions {
    pub structures: Vec<StructureGenerator>,
    pub structure_assets: Vec<StructureAsset>,
    pub country_cache: RwLock<HashMap<IVec2, Arc<RwLock<Option<Arc<CountryCache>>>>>>,
}

pub struct StructureAsset(Vec<Vec<Vec<BlockType>>>);

#[derive(Clone)]
pub struct CountryCache {
    pub country_pos: IVec2,
    pub grass_color: BlockType,
    pub start_location: [f32; 2],
    pub end_location: [f32; 2],
    pub path: Vec<IVec2>
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AStarCandidate {
    estimated_weight: i32,
    real_weight: i32,
    state: IVec2,
    direction: IVec2,
}

impl PartialOrd for AStarCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AStarCandidate {
    fn cmp(&self, other: &Self) -> Ordering {
        other.estimated_weight.cmp(&self.estimated_weight)
    }
}

impl CountryCache {
    pub fn new(country_pos: IVec2) -> Self {
        let mut rng = rand::thread_rng();

        let min_offset = 100i32;

        let start_x = rng.gen_range(min_offset..COUNTRY_SIZE as i32 - min_offset);
        let start_z = rng.gen_range(min_offset..COUNTRY_SIZE as i32 - min_offset);

        let mut end_x = start_x;
        let mut end_z = start_z;

        while (end_x - start_x).abs() < 5000 && (end_z - start_z).abs() < 5000 {
            end_x = rng.gen_range(min_offset..COUNTRY_SIZE as i32 - min_offset);
            end_z = rng.gen_range(min_offset..COUNTRY_SIZE as i32 - min_offset);
        }

        let path_finding_lod = ChunkLod::Sixtyfourth;

        let terrain_noise = get_terrain_noise(path_finding_lod);

        let start_pos = IVec2::new(start_x, start_z) / path_finding_lod.multiplier_i32();
        let end_pos = IVec2::new(end_x, end_z) / path_finding_lod.multiplier_i32();

        let get_terrain_height = |pos: IVec2| -> f64 {
            terrain_noise.get((pos * path_finding_lod.multiplier_i32() + country_pos * COUNTRY_SIZE as i32).to_array()) * path_finding_lod.multiplier_i32() as f64
        };

        let distance_to_end = |pos: IVec2| -> i32 {
            let diff = (end_pos - pos).abs();
            let smaller = if diff.x < diff.y { diff.x } else { diff.y };
            let bigger = if diff.x > diff.y { diff.x } else { diff.y };
            bigger * 10 + smaller * 4
        };

        let neighbours = |pos: IVec2| -> [(IVec2, i32); 8] {
            [
                (pos + IVec2::new(1, 0), 10),
                (pos + IVec2::new(0, 1), 10),
                (pos + IVec2::new(-1, 0), 10),
                (pos + IVec2::new(0, -1), 10),
                (pos + IVec2::new(1, 1), 14),
                (pos + IVec2::new(-1, 1), 14),
                (pos + IVec2::new(-1, -1), 14),
                (pos + IVec2::new(1, -1), 14),
            ]
        };

        let max_size = COUNTRY_SIZE as i32 / path_finding_lod.multiplier_i32();

        let mut queue = BinaryHeap::new();
        let mut previous = HashMap::new();
        let mut weights = HashMap::new();

        weights.insert(start_pos, 0);
        queue.push(AStarCandidate {
            estimated_weight: distance_to_end(start_pos),
            real_weight: 0,
            state: start_pos,
            direction: IVec2::ZERO,
        });

        info!("start_pos: {start_pos}, end_pos: {end_pos}");

        while let Some(AStarCandidate {
            estimated_weight: _,
            real_weight,
            state: current,
            direction: current_direction,
        }) = queue.pop() {
            if current == end_pos {
                break;
            }

            let current_height = get_terrain_height(current);

            for (next, weight) in neighbours(current) {
                if next.x < 0 || next.y < 0 || next.x >= max_size || next.y >= max_size {
                    continue;
                }

                let direction = next - current;
                let direction_difference = (direction - current_direction).abs();
                let direction_cost = direction_difference.x + direction_difference.y;

                let next_height = get_terrain_height(next);

                let x_neighbour = get_terrain_height(next + if next.x == 0 { IVec2::X } else { IVec2::NEG_X });
                let y_neighbour = get_terrain_height(next + if next.y == 0 { IVec2::Y } else { IVec2::NEG_Y });
                let total_steepness = (next_height - x_neighbour).abs().max((next_height - y_neighbour).abs()) / path_finding_lod.multiplier_i32() as f64;
                //info!("steepness: {total_steepness}");

                let height_difference = (current_height - next_height).abs() / path_finding_lod.multiplier_i32() as f64;
                if height_difference > 0.6 || direction_cost > 1 || total_steepness > 2.5 {
                    continue;
                }

                let real_weight = real_weight + weight + ((total_steepness * 0.5).max(0.) * 10.0) as i32;
                if weights
                    .get(&next)
                    .map(|&weight| real_weight < weight)
                    .unwrap_or(true)
                {
                    let estimated_weight = real_weight + distance_to_end(next);
                    weights.insert(next, real_weight);
                    queue.push(AStarCandidate {
                        estimated_weight,
                        real_weight,
                        state: next,
                        direction
                    });
                    previous.insert(next, current);
                }
            }
        }

        let path = if previous.get(&end_pos).is_some() {
            let mut current = end_pos;
            let mut path = vec![];

            let mut direction = IVec2::new(0, 0);

            while current != start_pos {
                let prev = previous
                    .get(&current)
                    .copied()
                    .expect("We reached the target, but are unable to reconsistute the path");

                let dir = prev - current;

                if dir != direction {
                    path.push(current * path_finding_lod.multiplier_i32());
                }

                direction = dir;

                current = prev;
            }
            path.push(current * path_finding_lod.multiplier_i32());
            path
        } else {
            info!("NO PATH COULD BE CREATED!");
            vec![]
        };

        info!("DONE");

        Self {
            country_pos,
            grass_color: Custom(rng.gen(), rng.gen(), rng.gen()),
            start_location: [start_x as f32, start_z as f32],
            end_location: [end_x as f32, end_z as f32],
            path,
        }
    }
}

pub const COUNTRY_SIZE: usize = 2usize.pow(16);


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
        result[voxel.point.x as usize][voxel.point.z as usize][voxel.point.y as usize] = Custom(color.r, color.g, color.b);
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