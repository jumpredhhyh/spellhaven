use crate::utils::div_floor;
use crate::world_generation::chunk_generation::noise::fractal_open_simplex::FractalOpenSimplex;
use crate::world_generation::chunk_generation::noise::roughness::Roughness;
use crate::world_generation::chunk_generation::{BlockType, CHUNK_SIZE, VOXEL_SIZE};
use crate::world_generation::chunk_loading::country_cache::{
    CountryCache, Path, PathLine, COUNTRY_SIZE,
};
use crate::world_generation::generation_options::GenerationOptions;
use crate::world_generation::voxel_world::ChunkLod;
use bevy::math::IVec2;
use bevy::prelude::Vec2;
use bracket_noise::prelude::FastNoise;
use noise::core::worley::distance_functions::{
    chebyshev, euclidean, euclidean_squared, manhattan, quadratic,
};
use noise::core::worley::ReturnType;
use noise::{
    Add, Clamp, Constant, Fbm, Min, MultiFractal, Multiply, NoiseFn, Perlin, ScalePoint, Seedable,
    Turbulence, Worley,
};
use rand::prelude::StdRng;
use rand::{Rng, SeedableRng};
use std::sync::Arc;

pub struct StructureGenerator {
    pub model: Arc<Vec<Vec<Vec<BlockType>>>>,
    pub model_size: [i32; 3],
    pub noise: FastNoise,
    pub generation_size: [i32; 2],
    pub grid_offset: [i32; 2],
    pub generate_debug_blocks: bool,
    pub debug_rgb_multiplier: [f32; 3],
    //pub height_offset: i32
}

pub fn generate_voxels(
    position: [i32; 3],
    generation_options: &GenerationOptions,
    chunk_lod: ChunkLod,
    country_cache: &CountryCache,
) -> (
    [[[BlockType; CHUNK_SIZE[2] + 2]; CHUNK_SIZE[1] + 2]; CHUNK_SIZE[0] + 2],
    i32,
    bool,
) {
    let mut blocks = [[[BlockType::Air; CHUNK_SIZE[2] + 2]; CHUNK_SIZE[1] + 2]; CHUNK_SIZE[0] + 2];
    //let value_noise = Fbm::<Perlin>::new(2).set_frequency(0.5f64.powi(12));

    let terrain_noise = get_terrain_noise(chunk_lod, generation_options);

    let mut terrain_height = [[0f32; CHUNK_SIZE[0] + 2]; CHUNK_SIZE[0] + 2];
    let mut terrain_steepness = [[0f32; CHUNK_SIZE[0]]; CHUNK_SIZE[0]];
    get_noise_map(
        IVec2::new(position[0], position[2])
            * IVec2::new(CHUNK_SIZE[0] as i32, CHUNK_SIZE[2] as i32),
        chunk_lod.multiplier_i32(),
        &terrain_noise,
        &mut terrain_height,
    );
    get_steepness_map(&mut terrain_steepness, &terrain_height);

    let min_height = (get_min_in_noise_map(&terrain_height) as i32).max(2) - 2
        + position[1] * CHUNK_SIZE[1] as i32
        - 10 / chunk_lod.multiplier_i32();

    let mut generate_more: bool = false;

    let all_paths = vec![
        &country_cache.this_path_cache.paths,
        &country_cache.bottom_path_cache.paths,
        &country_cache.left_path_cache.paths,
    ];

    for x in 0..CHUNK_SIZE[0] + 2 {
        for z in 0..CHUNK_SIZE[2] + 2 {
            let total_x =
                position[0] * CHUNK_SIZE[0] as i32 + x as i32 * chunk_lod.multiplier_i32();
            let total_z =
                position[2] * CHUNK_SIZE[2] as i32 + z as i32 * chunk_lod.multiplier_i32();

            //let dryness = value_noise.get([total_x as f64, total_z as f64]);
            //let mountain = mountain_noise.get([total_x as f64, total_z as f64]);

            let steepness = if x > 0 && z > 0 && x <= CHUNK_SIZE[0] && z <= CHUNK_SIZE[2] {
                terrain_steepness[x - 1][z - 1]
            } else {
                0.
            };

            let mut noise_height = terrain_height[x][z];

            let is_snow = noise_height * chunk_lod.multiplier_f32() > 3500. / VOXEL_SIZE;
            let is_grass_steep = if is_snow {
                steepness < 1.2
            } else {
                steepness < 0.8
            };

            let (mut path_distance, closest_point_on_path, _, line) =
                get_min_distance_to_path(IVec2::new(total_x, total_z), &all_paths, IVec2::ONE * 15);
            let is_path = path_distance <= 8.75;

            path_distance /= 10.;

            if path_distance <= 1.65 {
                let path_start_height =
                    terrain_noise.get(line.unwrap().start.as_dvec2().to_array()) as f32;
                let path_end_height =
                    terrain_noise.get(line.unwrap().end.as_dvec2().to_array()) as f32;
                let path_height = lerp(
                    path_start_height,
                    path_end_height,
                    line.unwrap().get_progress_on_line(closest_point_on_path),
                );

                let closest_point_height =
                    terrain_noise.get(closest_point_on_path.as_dvec2().to_array()) as f32;
                let closest_point_height = lerp(closest_point_height, noise_height, 0.5);

                let path_height = lerp(closest_point_height, path_height, 0.5);

                noise_height = lerp(
                    noise_height,
                    path_height,
                    (1.65 - path_distance.powi(2)).clamp(0., 1.),
                )
                .max(noise_height - 10.);
            }

            for y in min_height as usize
                ..noise_height.min((CHUNK_SIZE[1] + 2 + min_height as usize) as f32) as usize
            {
                if y == CHUNK_SIZE[1] + 1 + min_height as usize {
                    generate_more = true;
                }
                blocks[x][y - min_height as usize][z] = if is_path {
                    BlockType::Path
                } else {
                    if is_grass_steep && y + 1 == noise_height.floor() as usize {
                        if is_snow {
                            BlockType::Snow
                        } else {
                            BlockType::Grass
                        }
                    } else {
                        BlockType::Stone
                    }
                };
            }

            for structure in &generation_options.structures {
                let structure_offset_x = div_floor(
                    total_x + structure.grid_offset[0],
                    structure.generation_size[0],
                );
                let structure_offset_z = div_floor(
                    total_z + structure.grid_offset[1],
                    structure.generation_size[1],
                );
                let structure_value = structure
                    .noise
                    .get_noise(structure_offset_x as f32, structure_offset_z as f32)
                    * 0.5
                    + 0.5;
                if structure.generate_debug_blocks {
                    let top_terrain = (noise_height.min(CHUNK_SIZE[1] as f32 + min_height as f32)
                        as i32
                        - min_height.min(noise_height as i32))
                    .max(1) as usize
                        - 1;
                    let current_color = match blocks[x][top_terrain][z] {
                        BlockType::StructureDebug(r, g, b) => (r, g, b),
                        _ => (0u8, 0u8, 0u8),
                    };
                    blocks[x][top_terrain][z] = BlockType::StructureDebug(
                        ((structure_value) * structure.debug_rgb_multiplier[0] * 255.) as u8
                            + current_color.0,
                        ((structure_value) * structure.debug_rgb_multiplier[1] * 255.) as u8
                            + current_color.1,
                        ((structure_value) * structure.debug_rgb_multiplier[2] * 255.) as u8
                            + current_color.2,
                    )
                }
                let mut rand = StdRng::seed_from_u64((structure_value.abs() * 10000.) as u64);

                if structure_value > 0. {
                    let random_x =
                        rand.gen_range(0..=structure.generation_size[0] - structure.model_size[0]);
                    let random_z =
                        rand.gen_range(0..=structure.generation_size[1] - structure.model_size[2]);

                    let structure_x: i32 = (total_x + structure.grid_offset[0]
                        - structure_offset_x * structure.generation_size[0])
                        .abs()
                        - random_x;
                    let structure_z: i32 = (total_z + structure.grid_offset[1]
                        - structure_offset_z * structure.generation_size[1])
                        .abs()
                        - random_z;

                    if structure_x < 0
                        || structure_z < 0
                        || structure_x >= structure.model_size[0]
                        || structure_z >= structure.model_size[2]
                    {
                        continue;
                    }

                    let structure_noise_height_x = structure_offset_x
                        * structure.generation_size[0]
                        + (structure.model_size[0] / 2)
                        - structure.grid_offset[0]
                        + random_x;
                    let structure_noise_height_z = structure_offset_z
                        * structure.generation_size[1]
                        + (structure.model_size[2] / 2)
                        - structure.grid_offset[1]
                        + random_z;

                    let structure_center: IVec2 =
                        [structure_noise_height_x, structure_noise_height_z].into();

                    let country_bounds_check =
                        structure_center - country_cache.country_pos * COUNTRY_SIZE as i32;
                    if country_bounds_check.x >= 0
                        && country_bounds_check.y >= 0
                        && country_bounds_check.x < COUNTRY_SIZE as i32 - 1
                        && country_bounds_check.y < COUNTRY_SIZE as i32 - 1
                    {
                        let (a, _, _, _) = get_min_distance_to_path(
                            structure_center,
                            &all_paths,
                            IVec2::new(structure.model_size[0] / 2, structure.model_size[2] / 2)
                                + IVec2::ONE * 10,
                        );

                        if (a as i32) < structure.model_size[0] / 2 + structure.model_size[1] / 2 {
                            continue;
                        }
                    }

                    let noise_height = terrain_noise.get([
                        structure_noise_height_x as f64,
                        structure_noise_height_z as f64,
                    ]);

                    for (index, sub_structure) in
                        structure.model[structure_x as usize].iter().enumerate()
                    {
                        if (index as i32
                            + (noise_height * chunk_lod.multiplier_i32() as f64) as i32)
                            % chunk_lod.multiplier_i32()
                            != 0
                        {
                            continue;
                        }
                        let chunk_index = index / chunk_lod.multiplier_i32() as usize;
                        if (noise_height as i32 - min_height + chunk_index as i32) < 0 {
                            continue;
                        }
                        let structure_block = sub_structure[structure_z as usize];
                        if structure_block == BlockType::Air {
                            continue;
                        }
                        if noise_height as usize + chunk_index - min_height as usize
                            >= CHUNK_SIZE[1] + 2
                        {
                            generate_more = true;
                            break;
                        }
                        blocks[x][noise_height as usize + chunk_index - min_height as usize][z] =
                            structure_block;
                    }
                }
            }
        }
    }

    (blocks, min_height, generate_more)
}

pub fn get_terrain_noise(
    chunk_lod: ChunkLod,
    generation_options: &GenerationOptions,
) -> Add<
    f64,
    Multiply<
        f64,
        Add<
            f64,
            Add<
                f64,
                Add<
                    f64,
                    FractalOpenSimplex<Roughness>,
                    Multiply<
                        f64,
                        Multiply<
                            f64,
                            Clamp<f64, ScalePoint<Perlin>, 2>,
                            Clamp<
                                f64,
                                Multiply<
                                    f64,
                                    Add<
                                        f64,
                                        Multiply<f64, Turbulence<Worley, Perlin>, Constant, 2>,
                                        Constant,
                                        2,
                                    >,
                                    Constant,
                                    2,
                                >,
                                2,
                            >,
                            2,
                        >,
                        Constant,
                        2,
                    >,
                    2,
                >,
                FractalOpenSimplex<Roughness>,
                2,
            >,
            Constant,
            2,
        >,
        Constant,
        2,
    >,
    Constant,
    2,
> {
    let mut rng = StdRng::seed_from_u64(generation_options.seed + 1);

    Add::new(
        Multiply::new(
            Add::new(
                Add::new(
                    Add::new(
                        FractalOpenSimplex::new(
                            rng.gen(),
                            0.5f64.powi(10),
                            256.,
                            7,
                            2.,
                            0.5,
                            Roughness::new(1, 0.5f64.powi(10), 0.2),
                        ),
                        Multiply::new(
                            Multiply::new(
                                Clamp::new(
                                    ScalePoint::new(Perlin::new(rng.gen()))
                                        .set_scale(0.5f64.powi(15)),
                                )
                                .set_bounds(0., 1.),
                                Clamp::new(Multiply::new(
                                    Add::new(
                                        Multiply::new(
                                            Turbulence::<Worley, Perlin>::new(
                                                Worley::new(rng.gen())
                                                    .set_frequency(0.5f64.powi(13))
                                                    .set_distance_function(&euclidean)
                                                    .set_return_type(ReturnType::Distance),
                                            )
                                            .set_frequency(0.5f64.powi(10))
                                            .set_power(300.)
                                            .set_roughness(5)
                                            .set_seed(rng.gen()),
                                            Constant::new(-1.),
                                        ),
                                        Constant::new(1.),
                                    ),
                                    Constant::new(0.5),
                                ))
                                .set_bounds(0., 1.),
                            ),
                            Constant::new(10000.),
                        ),
                    ),
                    FractalOpenSimplex::new(
                        rng.gen(),
                        0.5f64.powi(15),
                        4096.,
                        7,
                        2.,
                        0.5,
                        Roughness::new(1, 0.5f64.powi(13), 0.2),
                    ),
                ),
                Constant::new(-1.),
            ),
            Constant::new(1. / chunk_lod.multiplier_f32() as f64),
        ),
        Constant::new(1. + 10. / chunk_lod.multiplier_i32() as f64),
    )
}

pub fn get_noise_map<const SIZE: usize, T: From<i32>, F: NoiseFn<T, 2>>(
    position: IVec2,
    zoom_multiplier: i32,
    noise_fn: &F,
    array: &mut [[f32; SIZE]; SIZE],
) {
    for x in 0..SIZE {
        for z in 0..SIZE {
            let total = position + IVec2::new(x as i32, z as i32) * zoom_multiplier;
            array[x][z] = noise_fn.get([total.x.into(), total.y.into()]) as f32;
        }
    }
}

pub fn get_steepness_map<const SIZE: usize, const SIZE2: usize>(
    array: &mut [[f32; SIZE]; SIZE],
    noise_map: &[[f32; SIZE2]; SIZE2],
) {
    for x in 0..SIZE {
        for z in 0..SIZE {
            let steepness_x = ((noise_map[x][z] - noise_map[x + 2][z]) / 2.).abs();
            let steepness_z = ((noise_map[x][z] - noise_map[x][z + 2]) / 2.).abs();
            array[x][z] = (steepness_x + steepness_z) / 2.;
        }
    }
}

fn get_min_in_noise_map<const SIZE: usize, T: PartialOrd + Copy>(map: &[[T; SIZE]; SIZE]) -> T {
    let mut min = map[0][0];

    for x in 0..SIZE {
        for z in 0..SIZE {
            let current = map[x][z];
            if current < min {
                min = current;
            }
        }
    }

    min
}

fn get_min_distance_to_path<'a>(
    pos: IVec2,
    paths_list: &'a Vec<&'a Vec<Path>>,
    margin: IVec2,
) -> (f32, IVec2, Vec2, Option<&'a PathLine>) {
    let mut min: Option<f32> = None;
    let mut closest_point_total = IVec2::ZERO;
    let mut path_dir = Vec2::ZERO;
    let mut end_path = None;

    for paths in paths_list {
        for path in *paths {
            if !path.is_in_box(pos, margin) {
                continue;
            }

            for line in &path.lines {
                if !line.is_in_box(pos, margin) {
                    continue;
                }

                if let Some((closest_point, closest_path_dir)) =
                    line.closest_point_on_path(pos, margin)
                {
                    let distance = closest_point.distance(pos.as_vec2());
                    match min {
                        None => {
                            min = Some(distance);
                            closest_point_total = closest_point.as_ivec2();
                            path_dir = closest_path_dir;
                            end_path = Some(line);
                        }
                        Some(current_min) => {
                            if distance < current_min {
                                min = Some(distance);
                                closest_point_total = closest_point.as_ivec2();
                                path_dir = closest_path_dir;
                                end_path = Some(line);
                            }
                        }
                    }
                }
            }
        }
    }

    (
        min.unwrap_or(f32::INFINITY),
        closest_point_total,
        path_dir,
        end_path,
    )
}

fn lerp(a: f32, b: f32, f: f32) -> f32 {
    a + f * (b - a)
}
