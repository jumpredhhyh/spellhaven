use crate::world_generation::chunk_generation::{BlockType, CHUNK_SIZE, VOXEL_SIZE};
use crate::world_generation::voxel_world::ChunkLod;
use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use rand::Rng;

use super::voxel_types::VoxelData;

pub fn generate_mesh(
    blocks: &VoxelData,
    min_height: i32,
    chunk_lod: ChunkLod,
) -> Option<(Mesh, Vec<Vec3>, Vec<[u32; 3]>)> {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut triangles: Vec<[u32; 3]> = Vec::new();
    // let mut colors: Vec<[f32; 4]> = Vec::new();

    let mut rng = rand::thread_rng();

    for x in 1..CHUNK_SIZE[0] + 1 {
        for y in 1..CHUNK_SIZE[1] + 1 {
            for z in 1..CHUNK_SIZE[2] + 1 {
                if blocks.is_air(x, y, z) || all_neighbours([x, y, z], &blocks) {
                    continue;
                }

                let x_pos = x as f32;
                let y_pos = y as f32;
                let z_pos = z as f32;
                // let mut color = blocks[x][y][z].get_color();
                // color[0] = color[0] * rng.gen_range(0.9..1.);
                // color[1] = color[1] * rng.gen_range(0.9..1.);
                // color[2] = color[2] * rng.gen_range(0.9..1.);

                if blocks.is_air(x, y + 1, z) {
                    let positions_count = positions.len() as u32;

                    // let aos = [
                    //     calculate_ambient_occlusion(
                    //         blocks[x - 1][y + 1][z] != BlockType::Air,
                    //         blocks[x][y + 1][z - 1] != BlockType::Air,
                    //         blocks[x - 1][y + 1][z - 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x + 1][y + 1][z] != BlockType::Air,
                    //         blocks[x][y + 1][z - 1] != BlockType::Air,
                    //         blocks[x + 1][y + 1][z - 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x + 1][y + 1][z] != BlockType::Air,
                    //         blocks[x][y + 1][z + 1] != BlockType::Air,
                    //         blocks[x + 1][y + 1][z + 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x - 1][y + 1][z] != BlockType::Air,
                    //         blocks[x][y + 1][z + 1] != BlockType::Air,
                    //         blocks[x - 1][y + 1][z + 1] != BlockType::Air,
                    //     ),
                    // ];

                    // add_colors(&mut colors, color, &aos);

                    // let rotate_quad = (aos[1] + aos[3]) < (aos[0] + aos[2]);

                    let rotate_quad = false;

                    positions.extend_from_slice(&[
                        [x_pos - 0.5, y_pos + 0.5, z_pos - 0.5],
                        [x_pos + 0.5, y_pos + 0.5, z_pos - 0.5],
                        [x_pos + 0.5, y_pos + 0.5, z_pos + 0.5],
                        [x_pos - 0.5, y_pos + 0.5, z_pos + 0.5],
                    ]);

                    normals.extend_from_slice(&[
                        [0., 1., 0.],
                        [0., 1., 0.],
                        [0., 1., 0.],
                        [0., 1., 0.],
                    ]);

                    triangles.extend_from_slice(&[
                        [
                            positions_count + 0,
                            positions_count + (if rotate_quad { 2 } else { 3 }),
                            positions_count + 1,
                        ],
                        [
                            positions_count + (if rotate_quad { 0 } else { 1 }),
                            positions_count + 3,
                            positions_count + 2,
                        ],
                    ]);
                }

                if blocks.is_air(x, y - 1, z) {
                    let positions_count = positions.len() as u32;

                    // let aos = [
                    //     calculate_ambient_occlusion(
                    //         blocks[x - 1][y - 1][z] != BlockType::Air,
                    //         blocks[x][y - 1][z - 1] != BlockType::Air,
                    //         blocks[x - 1][y - 1][z - 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x + 1][y - 1][z] != BlockType::Air,
                    //         blocks[x][y - 1][z - 1] != BlockType::Air,
                    //         blocks[x + 1][y - 1][z - 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x + 1][y - 1][z] != BlockType::Air,
                    //         blocks[x][y - 1][z + 1] != BlockType::Air,
                    //         blocks[x + 1][y - 1][z + 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x - 1][y - 1][z] != BlockType::Air,
                    //         blocks[x][y - 1][z + 1] != BlockType::Air,
                    //         blocks[x - 1][y - 1][z + 1] != BlockType::Air,
                    //     ),
                    // ];

                    // add_colors(&mut colors, color, &aos);

                    // let rotate_quad = (aos[1] + aos[3]) < (aos[0] + aos[2]);

                    let rotate_quad = false;

                    positions.extend_from_slice(&[
                        [x_pos - 0.5, y_pos - 0.5, z_pos - 0.5],
                        [x_pos + 0.5, y_pos - 0.5, z_pos - 0.5],
                        [x_pos + 0.5, y_pos - 0.5, z_pos + 0.5],
                        [x_pos - 0.5, y_pos - 0.5, z_pos + 0.5],
                    ]);

                    normals.extend_from_slice(&[
                        [0., -1., 0.],
                        [0., -1., 0.],
                        [0., -1., 0.],
                        [0., -1., 0.],
                    ]);

                    triangles.extend_from_slice(&[
                        [
                            positions_count + 0,
                            positions_count + 1,
                            positions_count + (if rotate_quad { 2 } else { 3 }),
                        ],
                        [
                            positions_count + (if rotate_quad { 0 } else { 1 }),
                            positions_count + 2,
                            positions_count + 3,
                        ],
                    ]);
                }

                if blocks.is_air(x + 1, y, z) {
                    let positions_count = positions.len() as u32;

                    // let aos = [
                    //     calculate_ambient_occlusion(
                    //         blocks[x + 1][y - 1][z] != BlockType::Air,
                    //         blocks[x + 1][y][z - 1] != BlockType::Air,
                    //         blocks[x + 1][y - 1][z - 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x + 1][y - 1][z] != BlockType::Air,
                    //         blocks[x + 1][y][z + 1] != BlockType::Air,
                    //         blocks[x + 1][y - 1][z + 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x + 1][y + 1][z] != BlockType::Air,
                    //         blocks[x + 1][y][z + 1] != BlockType::Air,
                    //         blocks[x + 1][y + 1][z + 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x + 1][y + 1][z] != BlockType::Air,
                    //         blocks[x + 1][y][z - 1] != BlockType::Air,
                    //         blocks[x + 1][y + 1][z - 1] != BlockType::Air,
                    //     ),
                    // ];

                    // add_colors(&mut colors, color, &aos);

                    // let rotate_quad = (aos[1] + aos[3]) < (aos[0] + aos[2]);

                    let rotate_quad = false;

                    positions.extend_from_slice(&[
                        [x_pos + 0.5, y_pos - 0.5, z_pos - 0.5],
                        [x_pos + 0.5, y_pos - 0.5, z_pos + 0.5],
                        [x_pos + 0.5, y_pos + 0.5, z_pos + 0.5],
                        [x_pos + 0.5, y_pos + 0.5, z_pos - 0.5],
                    ]);

                    normals.extend_from_slice(&[
                        [1., 0., 0.],
                        [1., 0., 0.],
                        [1., 0., 0.],
                        [1., 0., 0.],
                    ]);

                    triangles.extend_from_slice(&[
                        [
                            positions_count + 0,
                            positions_count + (if rotate_quad { 2 } else { 3 }),
                            positions_count + 1,
                        ],
                        [
                            positions_count + (if rotate_quad { 0 } else { 1 }),
                            positions_count + 3,
                            positions_count + 2,
                        ],
                    ]);
                }

                if blocks.is_air(x - 1, y, z) {
                    let positions_count = positions.len() as u32;

                    // let aos = [
                    //     calculate_ambient_occlusion(
                    //         blocks[x - 1][y - 1][z] != BlockType::Air,
                    //         blocks[x - 1][y][z - 1] != BlockType::Air,
                    //         blocks[x - 1][y - 1][z - 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x - 1][y - 1][z] != BlockType::Air,
                    //         blocks[x - 1][y][z + 1] != BlockType::Air,
                    //         blocks[x - 1][y - 1][z + 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x - 1][y + 1][z] != BlockType::Air,
                    //         blocks[x - 1][y][z + 1] != BlockType::Air,
                    //         blocks[x - 1][y + 1][z + 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x - 1][y + 1][z] != BlockType::Air,
                    //         blocks[x - 1][y][z - 1] != BlockType::Air,
                    //         blocks[x - 1][y + 1][z - 1] != BlockType::Air,
                    //     ),
                    // ];

                    // add_colors(&mut colors, color, &aos);

                    // let rotate_quad = (aos[1] + aos[3]) < (aos[0] + aos[2]);

                    let rotate_quad = false;

                    positions.extend_from_slice(&[
                        [x_pos - 0.5, y_pos - 0.5, z_pos - 0.5],
                        [x_pos - 0.5, y_pos - 0.5, z_pos + 0.5],
                        [x_pos - 0.5, y_pos + 0.5, z_pos + 0.5],
                        [x_pos - 0.5, y_pos + 0.5, z_pos - 0.5],
                    ]);

                    normals.extend_from_slice(&[
                        [-1., 0., 0.],
                        [-1., 0., 0.],
                        [-1., 0., 0.],
                        [-1., 0., 0.],
                    ]);

                    triangles.extend_from_slice(&[
                        [
                            positions_count + 0,
                            positions_count + 1,
                            positions_count + (if rotate_quad { 2 } else { 3 }),
                        ],
                        [
                            positions_count + (if rotate_quad { 0 } else { 1 }),
                            positions_count + 2,
                            positions_count + 3,
                        ],
                    ]);
                }

                if blocks.is_air(x, y, z + 1) {
                    let positions_count = positions.len() as u32;

                    // let aos = [
                    //     calculate_ambient_occlusion(
                    //         blocks[x - 1][y][z + 1] != BlockType::Air,
                    //         blocks[x][y - 1][z + 1] != BlockType::Air,
                    //         blocks[x - 1][y - 1][z + 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x - 1][y][z + 1] != BlockType::Air,
                    //         blocks[x][y + 1][z + 1] != BlockType::Air,
                    //         blocks[x - 1][y + 1][z + 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x + 1][y][z + 1] != BlockType::Air,
                    //         blocks[x][y + 1][z + 1] != BlockType::Air,
                    //         blocks[x + 1][y + 1][z + 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x + 1][y][z + 1] != BlockType::Air,
                    //         blocks[x][y - 1][z + 1] != BlockType::Air,
                    //         blocks[x + 1][y - 1][z + 1] != BlockType::Air,
                    //     ),
                    // ];

                    // add_colors(&mut colors, color, &aos);

                    // let rotate_quad = (aos[1] + aos[3]) < (aos[0] + aos[2]);

                    let rotate_quad = false;

                    positions.extend_from_slice(&[
                        [x_pos - 0.5, y_pos - 0.5, z_pos + 0.5],
                        [x_pos - 0.5, y_pos + 0.5, z_pos + 0.5],
                        [x_pos + 0.5, y_pos + 0.5, z_pos + 0.5],
                        [x_pos + 0.5, y_pos - 0.5, z_pos + 0.5],
                    ]);

                    normals.extend_from_slice(&[
                        [0., 0., 1.],
                        [0., 0., 1.],
                        [0., 0., 1.],
                        [0., 0., 1.],
                    ]);

                    triangles.extend_from_slice(&[
                        [
                            positions_count + 0,
                            positions_count + (if rotate_quad { 2 } else { 3 }),
                            positions_count + 1,
                        ],
                        [
                            positions_count + (if rotate_quad { 0 } else { 1 }),
                            positions_count + 3,
                            positions_count + 2,
                        ],
                    ]);
                }

                if blocks.is_air(x, y, z - 1) {
                    let positions_count = positions.len() as u32;

                    // let aos = [
                    //     calculate_ambient_occlusion(
                    //         blocks[x - 1][y][z - 1] != BlockType::Air,
                    //         blocks[x][y - 1][z - 1] != BlockType::Air,
                    //         blocks[x - 1][y - 1][z - 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x - 1][y][z - 1] != BlockType::Air,
                    //         blocks[x][y + 1][z - 1] != BlockType::Air,
                    //         blocks[x - 1][y + 1][z - 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x + 1][y][z - 1] != BlockType::Air,
                    //         blocks[x][y + 1][z - 1] != BlockType::Air,
                    //         blocks[x + 1][y + 1][z - 1] != BlockType::Air,
                    //     ),
                    //     calculate_ambient_occlusion(
                    //         blocks[x + 1][y][z - 1] != BlockType::Air,
                    //         blocks[x][y - 1][z - 1] != BlockType::Air,
                    //         blocks[x + 1][y - 1][z - 1] != BlockType::Air,
                    //     ),
                    // ];

                    // add_colors(&mut colors, color, &aos);

                    // let rotate_quad = (aos[1] + aos[3]) < (aos[0] + aos[2]);

                    let rotate_quad = false;

                    positions.extend_from_slice(&[
                        [x_pos - 0.5, y_pos - 0.5, z_pos - 0.5],
                        [x_pos - 0.5, y_pos + 0.5, z_pos - 0.5],
                        [x_pos + 0.5, y_pos + 0.5, z_pos - 0.5],
                        [x_pos + 0.5, y_pos - 0.5, z_pos - 0.5],
                    ]);

                    normals.extend_from_slice(&[
                        [0., 0., -1.],
                        [0., 0., -1.],
                        [0., 0., -1.],
                        [0., 0., -1.],
                    ]);

                    triangles.extend_from_slice(&[
                        [
                            positions_count + 0,
                            positions_count + 1,
                            positions_count + (if rotate_quad { 2 } else { 3 }),
                        ],
                        [
                            positions_count + (if rotate_quad { 0 } else { 1 }),
                            positions_count + 2,
                            positions_count + 3,
                        ],
                    ]);
                }
            }
        }
    }

    if triangles.is_empty() {
        return None;
    }

    for position in positions.iter_mut() {
        position[0] = (position[0] - 0.5) * VOXEL_SIZE * chunk_lod.multiplier_f32() + 0.5;
        position[1] =
            (position[1] + min_height as f32 - 0.5) * VOXEL_SIZE * chunk_lod.multiplier_f32() + 0.5;
        position[2] = (position[2] - 0.5) * VOXEL_SIZE * chunk_lod.multiplier_f32() + 0.5;
    }

    let mut mesh_triangles: Vec<u32> = Vec::new();

    for triangle in &triangles {
        mesh_triangles.push(triangle[0]);
        mesh_triangles.push(triangle[1]);
        mesh_triangles.push(triangle[2]);
    }

    let collider_positions = if chunk_lod == ChunkLod::Full {
        positions
            .clone()
            .iter()
            .map(|position| Vec3::new(position[0], position[1], position[2]))
            .collect()
    } else {
        Vec::new()
    };

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    // mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(mesh_triangles));

    Some((
        mesh,
        collider_positions,
        if chunk_lod == ChunkLod::Full {
            triangles
        } else {
            Vec::new()
        },
    ))
}

fn all_neighbours(pos: [usize; 3], blocks: &VoxelData) -> bool {
    if blocks.is_air(pos[0], pos[1] + 1, pos[2]) {
        return false;
    }
    if blocks.is_air(pos[0], pos[1] - 1, pos[2]) {
        return false;
    }
    if blocks.is_air(pos[0] + 1, pos[1], pos[2]) {
        return false;
    }
    if blocks.is_air(pos[0] - 1, pos[1], pos[2]) {
        return false;
    }
    if blocks.is_air(pos[0], pos[1], pos[2] + 1) {
        return false;
    }
    if blocks.is_air(pos[0], pos[1], pos[2] - 1) {
        return false;
    }
    return true;
}

fn calculate_ambient_occlusion(side1: bool, side2: bool, corner: bool) -> f32 {
    if side1 && side2 {
        return 0.1;
    }
    if (side1 || side2) && corner {
        return 0.25;
    }
    if side1 || side2 || corner {
        return 0.5;
    }
    return 1.;
}

fn add_colors(
    colors: &mut Vec<[f32; 4]>,
    color: [f32; 4],
    ambient_occlusion_multipliers: &[f32; 4],
) {
    for i in 0..4 {
        colors.push([
            color[0] * ambient_occlusion_multipliers[i],
            color[1] * ambient_occlusion_multipliers[i],
            color[2] * ambient_occlusion_multipliers[i],
            color[3],
        ]);
    }
}
