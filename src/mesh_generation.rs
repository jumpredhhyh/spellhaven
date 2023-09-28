use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use rand::Rng;
use crate::chunk_generation::{BlockType, CHUNK_SIZE, VOXEL_SIZE};

pub fn generate_mesh(blocks: [[[BlockType; CHUNK_SIZE[2] + 2]; CHUNK_SIZE[1] + 2]; CHUNK_SIZE[0] + 2]) -> Option<(Mesh, Vec<Vec3>, Vec<[u32; 3]>)> {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut triangles: Vec<[u32; 3]> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();

    let mut rand = rand::thread_rng();

    for x in 1..CHUNK_SIZE[0] + 1 {
        for y in 1..CHUNK_SIZE[1] + 1 {
            for z in 1..CHUNK_SIZE[2] + 1 {
                if blocks[x][y][z] == BlockType::Air {
                    continue;
                }

                let x_pos = x as f32;
                let y_pos = y as f32;
                let z_pos = z as f32;
                let mut color = get_block_color(blocks[x][y][z]);
                color[0] = color[0] * rand.gen_range(0.8..1.);
                color[1] = color[1] * rand.gen_range(0.8..1.);
                color[2] = color[2] * rand.gen_range(0.8..1.);

                if blocks[x][y + 1][z] == BlockType::Air {
                    let positions_count = positions.len() as u32;

                    let aos = [
                        calculate_ambient_occlusion(blocks[x - 1][y + 1][z] != BlockType::Air, blocks[x][y + 1][z - 1] != BlockType::Air, blocks[x - 1][y + 1][z - 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x + 1][y + 1][z] != BlockType::Air, blocks[x][y + 1][z - 1] != BlockType::Air, blocks[x + 1][y + 1][z - 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x + 1][y + 1][z] != BlockType::Air, blocks[x][y + 1][z + 1] != BlockType::Air, blocks[x + 1][y + 1][z + 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x - 1][y + 1][z] != BlockType::Air, blocks[x][y + 1][z + 1] != BlockType::Air, blocks[x - 1][y + 1][z + 1] != BlockType::Air),
                    ];

                    add_colors(&mut colors, color, &aos);

                    let rotate_quad = (aos[1] + aos[3]) < (aos[0] + aos[2]);

                    positions.extend_from_slice(&[
                        [x_pos - 0.5, y_pos + 0.5, z_pos - 0.5],
                        [x_pos + 0.5, y_pos + 0.5, z_pos - 0.5],
                        [x_pos + 0.5, y_pos + 0.5, z_pos + 0.5],
                        [x_pos - 0.5, y_pos + 0.5, z_pos + 0.5]
                    ]);

                    triangles.extend_from_slice(&[[
                            positions_count + 0,
                            positions_count + (if rotate_quad {2} else {3}),
                            positions_count + 1,
                        ], [
                            positions_count + (if rotate_quad {0} else {1}),
                            positions_count + 3,
                            positions_count + 2
                    ]]);
                }

                if blocks[x][y - 1][z] == BlockType::Air {
                    let positions_count = positions.len() as u32;

                    let aos = [
                        calculate_ambient_occlusion(blocks[x - 1][y - 1][z] != BlockType::Air, blocks[x][y - 1][z - 1] != BlockType::Air, blocks[x - 1][y - 1][z - 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x + 1][y - 1][z] != BlockType::Air, blocks[x][y - 1][z - 1] != BlockType::Air, blocks[x + 1][y - 1][z - 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x + 1][y - 1][z] != BlockType::Air, blocks[x][y - 1][z + 1] != BlockType::Air, blocks[x + 1][y - 1][z + 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x - 1][y - 1][z] != BlockType::Air, blocks[x][y - 1][z + 1] != BlockType::Air, blocks[x - 1][y - 1][z + 1] != BlockType::Air),
                    ];

                    add_colors(&mut colors, color, &aos);

                    let rotate_quad = (aos[1] + aos[3]) < (aos[0] + aos[2]);

                    positions.extend_from_slice(&[
                        [x_pos - 0.5, y_pos - 0.5, z_pos - 0.5],
                        [x_pos + 0.5, y_pos - 0.5, z_pos - 0.5],
                        [x_pos + 0.5, y_pos - 0.5, z_pos + 0.5],
                        [x_pos - 0.5, y_pos - 0.5, z_pos + 0.5]
                    ]);

                    triangles.extend_from_slice(&[[
                        positions_count + 0,
                        positions_count + 1,
                        positions_count + (if rotate_quad {2} else {3}),
                    ], [
                        positions_count + (if rotate_quad {0} else {1}),
                        positions_count + 2,
                        positions_count + 3
                    ]]);
                }

                if blocks[x + 1][y][z] == BlockType::Air {
                    let positions_count = positions.len() as u32;

                    let aos = [
                        calculate_ambient_occlusion(blocks[x + 1][y - 1][z] != BlockType::Air, blocks[x + 1][y][z - 1] != BlockType::Air, blocks[x + 1][y - 1][z - 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x + 1][y - 1][z] != BlockType::Air, blocks[x + 1][y][z + 1] != BlockType::Air, blocks[x + 1][y - 1][z + 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x + 1][y + 1][z] != BlockType::Air, blocks[x + 1][y][z + 1] != BlockType::Air, blocks[x + 1][y + 1][z + 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x + 1][y + 1][z] != BlockType::Air, blocks[x + 1][y][z - 1] != BlockType::Air, blocks[x + 1][y + 1][z - 1] != BlockType::Air),
                    ];

                    add_colors(&mut colors, color, &aos);

                    let rotate_quad = (aos[1] + aos[3]) < (aos[0] + aos[2]);

                    positions.extend_from_slice(&[
                        [x_pos + 0.5, y_pos - 0.5, z_pos - 0.5],
                        [x_pos + 0.5, y_pos - 0.5, z_pos + 0.5],
                        [x_pos + 0.5, y_pos + 0.5, z_pos + 0.5],
                        [x_pos + 0.5, y_pos + 0.5, z_pos - 0.5]
                    ]);

                    triangles.extend_from_slice(&[[
                        positions_count + 0,
                        positions_count + (if rotate_quad {2} else {3}),
                        positions_count + 1,
                    ], [
                        positions_count + (if rotate_quad {0} else {1}),
                        positions_count + 3,
                        positions_count + 2
                    ]]);
                }

                if blocks[x - 1][y][z] == BlockType::Air {
                    let positions_count = positions.len() as u32;

                    let aos = [
                        calculate_ambient_occlusion(blocks[x - 1][y - 1][z] != BlockType::Air, blocks[x - 1][y][z - 1] != BlockType::Air, blocks[x - 1][y - 1][z - 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x - 1][y - 1][z] != BlockType::Air, blocks[x - 1][y][z + 1] != BlockType::Air, blocks[x - 1][y - 1][z + 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x - 1][y + 1][z] != BlockType::Air, blocks[x - 1][y][z + 1] != BlockType::Air, blocks[x - 1][y + 1][z + 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x - 1][y + 1][z] != BlockType::Air, blocks[x - 1][y][z - 1] != BlockType::Air, blocks[x - 1][y + 1][z - 1] != BlockType::Air),
                    ];

                    add_colors(&mut colors, color, &aos);

                    let rotate_quad = (aos[1] + aos[3]) < (aos[0] + aos[2]);

                    positions.extend_from_slice(&[
                        [x_pos - 0.5, y_pos - 0.5, z_pos - 0.5],
                        [x_pos - 0.5, y_pos - 0.5, z_pos + 0.5],
                        [x_pos - 0.5, y_pos + 0.5, z_pos + 0.5],
                        [x_pos - 0.5, y_pos + 0.5, z_pos - 0.5]
                    ]);

                    triangles.extend_from_slice(&[[
                        positions_count + 0,
                        positions_count + 1,
                        positions_count + (if rotate_quad {2} else {3}),
                    ], [
                        positions_count + (if rotate_quad {0} else {1}),
                        positions_count + 2,
                        positions_count + 3
                    ]]);
                }

                if blocks[x][y][z + 1] == BlockType::Air {
                    let positions_count = positions.len() as u32;

                    let aos = [
                        calculate_ambient_occlusion(blocks[x - 1][y][z + 1] != BlockType::Air, blocks[x][y - 1][z + 1] != BlockType::Air, blocks[x - 1][y - 1][z + 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x - 1][y][z + 1] != BlockType::Air, blocks[x][y + 1][z + 1] != BlockType::Air, blocks[x - 1][y + 1][z + 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x + 1][y][z + 1] != BlockType::Air, blocks[x][y + 1][z + 1] != BlockType::Air, blocks[x + 1][y + 1][z + 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x + 1][y][z + 1] != BlockType::Air, blocks[x][y - 1][z + 1] != BlockType::Air, blocks[x + 1][y - 1][z + 1] != BlockType::Air),
                    ];

                    add_colors(&mut colors, color, &aos);

                    let rotate_quad = (aos[1] + aos[3]) < (aos[0] + aos[2]);

                    positions.extend_from_slice(&[
                        [x_pos - 0.5, y_pos - 0.5, z_pos + 0.5],
                        [x_pos - 0.5, y_pos + 0.5, z_pos + 0.5],
                        [x_pos + 0.5, y_pos + 0.5, z_pos + 0.5],
                        [x_pos + 0.5, y_pos - 0.5, z_pos + 0.5]
                    ]);

                    triangles.extend_from_slice(&[[
                        positions_count + 0,
                        positions_count + (if rotate_quad {2} else {3}),
                        positions_count + 1,
                    ], [
                        positions_count + (if rotate_quad {0} else {1}),
                        positions_count + 3,
                        positions_count + 2
                    ]]);
                }

                if blocks[x][y][z - 1] == BlockType::Air {
                    let positions_count = positions.len() as u32;

                    let aos = [
                        calculate_ambient_occlusion(blocks[x - 1][y][z - 1] != BlockType::Air, blocks[x][y - 1][z - 1] != BlockType::Air, blocks[x - 1][y - 1][z - 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x - 1][y][z - 1] != BlockType::Air, blocks[x][y + 1][z - 1] != BlockType::Air, blocks[x - 1][y + 1][z - 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x + 1][y][z - 1] != BlockType::Air, blocks[x][y + 1][z - 1] != BlockType::Air, blocks[x + 1][y + 1][z - 1] != BlockType::Air),
                        calculate_ambient_occlusion(blocks[x + 1][y][z - 1] != BlockType::Air, blocks[x][y - 1][z - 1] != BlockType::Air, blocks[x + 1][y - 1][z - 1] != BlockType::Air),
                    ];

                    add_colors(&mut colors, color, &aos);

                    let rotate_quad = (aos[1] + aos[3]) < (aos[0] + aos[2]);

                    positions.extend_from_slice(&[
                        [x_pos - 0.5, y_pos - 0.5, z_pos - 0.5],
                        [x_pos - 0.5, y_pos + 0.5, z_pos - 0.5],
                        [x_pos + 0.5, y_pos + 0.5, z_pos - 0.5],
                        [x_pos + 0.5, y_pos - 0.5, z_pos - 0.5]
                    ]);

                    triangles.extend_from_slice(&[[
                        positions_count + 0,
                        positions_count + 1,
                        positions_count + (if rotate_quad {2} else {3}),
                    ], [
                        positions_count + (if rotate_quad {0} else {1}),
                        positions_count + 2,
                        positions_count + 3
                    ]]);
                }
            }
        }
    }

    for position in positions.iter_mut() {
        position[0] = position[0] * VOXEL_SIZE;
        position[1] = position[1] * VOXEL_SIZE;
        position[2] = position[2] * VOXEL_SIZE;
    }

    let mut mesh_triangles: Vec<u32> = Vec::new();

    for triangle in &triangles {
        mesh_triangles.push(triangle[0]);
        mesh_triangles.push(triangle[1]);
        mesh_triangles.push(triangle[2]);
    }

    let collider_positions = positions.clone().iter().map(|position| Vec3::new(position[0], position[1], position[2])).collect();

    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        positions
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.set_indices(Some(Indices::U32(mesh_triangles)));

    mesh.duplicate_vertices();

    mesh.compute_flat_normals();

    if triangles.is_empty() {
        return None;
    }

    Some((mesh, collider_positions, triangles))
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

fn add_colors(colors: &mut Vec<[f32; 4]>, color: [f32; 4], ambient_occlusion_multipliers: &[f32; 4]) {
    for i in 0..4 {
        colors.push([color[0] * ambient_occlusion_multipliers[i], color[1] * ambient_occlusion_multipliers[i], color[2] * ambient_occlusion_multipliers[i], color[3]]);
    }
}

fn get_block_color(block: BlockType) -> [f32; 4] {
    match block {
        BlockType::Air => [0., 0., 0., 0.],
        BlockType::Stone => [150. / 255., 160. / 255., 155. / 255., 1.],
        BlockType::Grass => [55. / 255., 195. /255., 95. / 255., 1.],
        BlockType::Gray(value) => [value as f32 / 255., value as f32 / 255., value as f32 / 255., 1.]
    }
}