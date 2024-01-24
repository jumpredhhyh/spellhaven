use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use bevy::log::info;
use bevy::math::IVec2;
use noise::NoiseFn;
use rand::Rng;
use crate::chunk_generation::BlockType;
use crate::chunk_generation::BlockType::Custom;
use crate::generation_options::{GenerationCacheItem, GenerationOptions};
use crate::voxel_generation::get_terrain_noise;
use crate::voxel_world::ChunkLod;

#[derive(Clone)]
pub struct CountryCache {
    pub country_pos: IVec2,
    pub grass_color: BlockType,
    pub structure_cache: Arc<StructureCache>,
    pub this_path_cache: Arc<PathCache>,
    pub bottom_path_cache: Arc<PathCache>,
    pub left_path_cache: Arc<PathCache>,
}

pub struct StructureCache {
    pub city_location: IVec2,
}

pub struct  PathCache {
    pub paths: Vec<Path>
}

pub struct Path {
    pub lines: Vec<PathLine>,
    pub box_pos_start: IVec2,
    pub box_pos_end: IVec2,
}

impl Path {
    pub fn is_in_box(&self, point: IVec2, margin: IVec2) -> bool {
        let bb_start = self.box_pos_start - margin;
        let bb_end = self.box_pos_end + margin;
        !(point.x < bb_start.x || point.x > bb_end.x || point.y < bb_start.y || point.y > bb_end.y)
    }
}

pub struct PathLine {
    pub start: IVec2,
    pub end: IVec2,
    pub box_pos_start: IVec2,
    pub box_pos_end: IVec2,
}

impl PathLine {
    fn new(start: IVec2, end: IVec2) -> Self {
        let box_pos_start = IVec2::new(start.x.min(end.x), start.y.min(end.y));
        let box_pos_end = IVec2::new(start.x.max(end.x), start.y.max(end.y));

        Self {
            start,
            end,
            box_pos_start,
            box_pos_end,
        }
    }

    pub fn is_in_box(&self, point: IVec2, margin: IVec2) -> bool {
        let bb_start = self.box_pos_start - margin;
        let bb_end = self.box_pos_end + margin;
        !(point.x < bb_start.x || point.x > bb_end.x || point.y < bb_start.y || point.y > bb_end.y)
    }
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

impl GenerationCacheItem<IVec2> for CountryCache {
    fn generate(key: IVec2, generation_options: Arc<GenerationOptions>) -> Self {
        let mut rng = rand::thread_rng();

        Self {
            country_pos: key,
            grass_color: Custom(rng.gen(), rng.gen(), rng.gen()),
            structure_cache: generation_options.structure_cache.get_cache_entry(key, generation_options.clone()),
            this_path_cache: generation_options.path_cache.get_cache_entry(key, generation_options.clone()),
            bottom_path_cache: generation_options.path_cache.get_cache_entry(key + IVec2::NEG_X, generation_options.clone()),
            left_path_cache: generation_options.path_cache.get_cache_entry(key + IVec2::NEG_Y, generation_options.clone()),
        }
    }
}

impl GenerationCacheItem<IVec2> for StructureCache {
    fn generate(key: IVec2, _generation_options: Arc<GenerationOptions>) -> Self {
        let mut rng = rand::thread_rng();

        let min_offset = 100i32;

        let city_x = rng.gen_range(min_offset..COUNTRY_SIZE as i32 - min_offset);
        let city_z = rng.gen_range(min_offset..COUNTRY_SIZE as i32 - min_offset);

        Self {
            city_location: IVec2::new(city_x, city_z) + key * COUNTRY_SIZE as i32
        }
    }
}

impl GenerationCacheItem<IVec2> for PathCache {
    fn generate(key: IVec2, generation_options: Arc<GenerationOptions>) -> Self {
        let top_country_pos = key + IVec2::X;
        let right_country_pos = key + IVec2::Y;

        let current_structure_cache = generation_options.structure_cache.get_cache_entry(key, generation_options.clone());
        let top_structure_cache = generation_options.structure_cache.get_cache_entry(top_country_pos, generation_options.clone());
        let right_structure_cache = generation_options.structure_cache.get_cache_entry(right_country_pos, generation_options.clone());

        let path_finding_lod = ChunkLod::OneTwentyEight;

        Self {
            paths: vec![
                PathCache::generate_path(current_structure_cache.city_location, top_structure_cache.city_location, [key, top_country_pos], path_finding_lod),
                PathCache::generate_path(current_structure_cache.city_location, right_structure_cache.city_location, [key, right_country_pos], path_finding_lod),
            ],
        }
    }
}

impl PathCache {
    fn generate_path(mut start_pos: IVec2, mut end_pos: IVec2, country_positions: [IVec2; 2], path_finding_lod: ChunkLod) -> Path {
        start_pos /= path_finding_lod.multiplier_i32();
        end_pos /= path_finding_lod.multiplier_i32();

        let terrain_noise = get_terrain_noise(path_finding_lod);

        let get_terrain_height = |pos: IVec2| -> f64 {
            terrain_noise.get((pos * path_finding_lod.multiplier_i32()).to_array()) * path_finding_lod.multiplier_i32() as f64
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

        let is_outside_of_countries = |pos: IVec2| -> bool {
            let pos = pos * path_finding_lod.multiplier_i32();
            let is_outside_first_country = pos.x < country_positions[0].x * COUNTRY_SIZE as i32 || pos.x >= (country_positions[0] + IVec2::X).x * COUNTRY_SIZE as i32 || pos.y < country_positions[0].y * COUNTRY_SIZE as i32 || pos.y >= (country_positions[0] + IVec2::Y).y * COUNTRY_SIZE as i32;
            let is_outside_second_country = pos.x < country_positions[1].x * COUNTRY_SIZE as i32 || pos.x >= (country_positions[1] + IVec2::X).x * COUNTRY_SIZE as i32 || pos.y < country_positions[1].y * COUNTRY_SIZE as i32 || pos.y >= (country_positions[1] + IVec2::Y).y * COUNTRY_SIZE as i32;
            is_outside_first_country && is_outside_second_country
        };

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
                if is_outside_of_countries(next) {
                    //info!("Outside of countries!");
                    continue;
                }

                let direction = next - current;
                let direction_difference = (direction - current_direction).abs();
                let direction_cost = direction_difference.x + direction_difference.y;

                let next_height = get_terrain_height(next);

                let x_neighbour = get_terrain_height(next + if next.x == 0 { IVec2::X } else { IVec2::NEG_X });
                let y_neighbour = get_terrain_height(next + if next.y == 0 { IVec2::Y } else { IVec2::NEG_Y });
                let total_steepness = (next_height - x_neighbour).abs().max((next_height - y_neighbour).abs()) / path_finding_lod.multiplier_i32() as f64;

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

        info!("DONE");

        if previous.get(&end_pos).is_some() {
            let mut min_x = 0;
            let mut min_y = 0;
            let mut max_x = 0;
            let mut max_y = 0;

            let mut check_min_max = |pos: IVec2| {
                min_x = min_x.min(pos.x);
                min_y = min_y.min(pos.y);
                max_x = max_x.max(pos.x);
                max_y = max_y.max(pos.y);
            };

            let mut current = end_pos;
            let mut path: Vec<PathLine> = vec![];

            let mut direction = IVec2::new(0, 0);

            while current != start_pos {
                let prev = previous
                    .get(&current)
                    .copied()
                    .expect("We reached the target, but are unable to reconsistute the path");

                let dir = prev - current;

                if dir != direction {
                    let start = path.last().map(|x| x.end).unwrap_or(prev * path_finding_lod.multiplier_i32());
                    let end = current * path_finding_lod.multiplier_i32();
                    path.push(PathLine::new(
                        start,
                        end
                    ));

                    check_min_max(start);
                    check_min_max(end);
                }

                direction = dir;

                current = prev;
            }
            let last = current * path_finding_lod.multiplier_i32();
            path.push(PathLine::new(
                path.last().map(|x| x.end).unwrap(),
                last
            ));

            check_min_max(last);

            Path {
                lines: path,
                box_pos_start: IVec2::new(min_x, min_y),
                box_pos_end: IVec2::new(max_x, max_y),
            }
        } else {
            info!("NO PATH COULD BE CREATED!");
            Path {
                lines: vec![],
                box_pos_start: Default::default(),
                box_pos_end: Default::default(),
            }
        }
    }
}

pub const COUNTRY_SIZE: usize = 2usize.pow(16);