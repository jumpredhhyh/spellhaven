use crate::{
    utils::{rotate_around, vec_round_to_int, RotationDirection},
    world_generation::chunk_generation::{BlockType, VOXEL_SIZE},
};
use bevy::math::{IVec3, Vec3};
use rand::{rngs::StdRng, Rng};
use std::ops::Range;

pub struct TreeLSystem;

pub enum LSystemEntryType {
    Stem,
    Branch {
        angle_x: f32,
        angle_z: f32,
        available_dirs: Directions,
    },
    Leaf,
}

pub struct LSystemEntry {
    pub pos: Vec3,
    pub thickness: f32,
    pub entry_type: LSystemEntryType,
}

bitflags::bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Directions: u8 {
        const XZ = 0x01;
        const NegXZ = 0x02;
        const XNegZ = 0x04;
        const NegXNegZ = 0x08;
    }
}

impl TreeLSystem {
    pub fn grow_new<const XSIZE: usize, const YSIZE: usize, const ZSIZE: usize>(
        rng: &mut StdRng,
    ) -> Vec<Vec<Vec<BlockType>>> {
        let pos = Vec3::new(XSIZE as f32 / 2., 0., ZSIZE as f32 / 2.);
        let pos_offset = Vec3 {
            x: rng.random(),
            y: 0.,
            z: rng.random(),
        };
        let mut start_state = create_branch_piece(&(pos + pos_offset), 0., 0., 2.0, rng);

        for _ in 0..3 {
            TreeLSystem::recurse_l_system(&mut start_state, rng);
        }
        TreeLSystem::add_leafs(&mut start_state);

        let mut voxel_grid = vec![];

        for x in 0..ZSIZE {
            voxel_grid.push(vec![]);
            for y in 0..YSIZE {
                voxel_grid[x].push(vec![]);
                for _ in 0..XSIZE {
                    voxel_grid[x][y].push(BlockType::Air);
                }
            }
        }

        start_state.iter().for_each(|entry| {
            let entry_pos = entry.pos;
            let center = vec_round_to_int(&entry_pos);
            let thickness = (entry.thickness / VOXEL_SIZE).ceil() as i32;

            for x in -thickness..thickness {
                for y in -thickness..thickness {
                    for z in -thickness..thickness {
                        let current_pos_i =
                            center + (Vec3::new(x as f32, y as f32, z as f32)).as_ivec3();
                        let current_pos = current_pos_i.as_vec3();

                        if current_pos_i.x < 0
                            || current_pos_i.x >= XSIZE as i32
                            || current_pos_i.y < 0
                            || current_pos_i.y >= YSIZE as i32
                            || current_pos_i.z < 0
                            || current_pos_i.z >= ZSIZE as i32
                        {
                            continue;
                        }

                        if current_pos.distance_squared(entry_pos)
                            < entry.thickness * entry.thickness / VOXEL_SIZE
                        {
                            voxel_grid[current_pos_i.x as usize][current_pos_i.y as usize]
                                [current_pos_i.z as usize] = match entry.entry_type {
                                LSystemEntryType::Leaf => BlockType::Grass(100),
                                _ => BlockType::Path,
                            }
                        }
                    }
                }
            }
        });

        voxel_grid
    }

    fn recurse_l_system(data: &mut Vec<LSystemEntry>, rng: &mut StdRng) {
        let mut i = 0usize;
        while i < data.len() {
            let entry = &data[i];
            match entry.entry_type {
                LSystemEntryType::Branch {
                    angle_x,
                    angle_z,
                    available_dirs: _,
                } => {
                    let random_range: Range<f32> = -45.0..45.0;
                    let new_thickness = (entry.thickness - 0.5).max(0.75);

                    let branches: Vec<LSystemEntry> = create_branch_piece(
                        &entry.pos,
                        angle_x + rng.random_range(random_range.clone()),
                        angle_z + rng.random_range(random_range.clone()),
                        new_thickness,
                        rng,
                    )
                    .into_iter()
                    .chain(create_branch_piece(
                        &entry.pos,
                        angle_x + rng.random_range(random_range.clone()),
                        angle_z + rng.random_range(random_range.clone()),
                        new_thickness,
                        rng,
                    ))
                    .chain(create_branch_piece(
                        &entry.pos,
                        angle_x + rng.random_range(random_range.clone()),
                        angle_z + rng.random_range(random_range.clone()),
                        new_thickness,
                        rng,
                    ))
                    .chain(create_branch_piece(
                        &entry.pos,
                        angle_x + rng.random_range(random_range.clone()),
                        angle_z + rng.random_range(random_range.clone()),
                        new_thickness,
                        rng,
                    ))
                    .chain(create_branch_piece(
                        &entry.pos,
                        angle_x + rng.random_range(random_range.clone()),
                        angle_z + rng.random_range(random_range.clone()),
                        new_thickness,
                        rng,
                    ))
                    .chain(create_branch_piece(
                        &entry.pos,
                        angle_x + rng.random_range(random_range.clone()),
                        angle_z + rng.random_range(random_range),
                        new_thickness,
                        rng,
                    ))
                    .collect();

                    let length = branches.len();

                    data.splice(i..i + 1, branches);
                    i += length
                }
                _ => {}
            }
            i += 1;
        }
    }

    fn add_leafs(data: &mut Vec<LSystemEntry>) {
        let mut i = 0usize;
        while i < data.len() {
            let entry = &data[i];
            match entry.entry_type {
                LSystemEntryType::Branch {
                    angle_x: _,
                    angle_z: _,
                    available_dirs: _,
                } => {
                    let branches: Vec<LSystemEntry> = vec![LSystemEntry {
                        pos: entry.pos,
                        entry_type: LSystemEntryType::Leaf,
                        thickness: 2.0,
                    }];

                    let length = branches.len();

                    data.splice(i..i + 1, branches);
                    i += length
                }
                _ => {}
            }
            i += 1;
        }
    }
}

fn create_branch_piece(
    pos: &Vec3,
    angle_x: f32,
    angle_z: f32,
    thickness: f32,
    rng: &mut StdRng,
) -> Vec<LSystemEntry> {
    let length: usize = (rng.random_range(3.0..6.0) / VOXEL_SIZE) as usize;

    let mut pieces = Vec::new();

    for i in 0..length {
        pieces.push(LSystemEntry {
            pos: rotate_around(
                &rotate_around(
                    &(*pos + (Vec3::Y * (i as f32))),
                    pos,
                    angle_z,
                    &RotationDirection::Z,
                ),
                pos,
                angle_x,
                &RotationDirection::X,
            ),
            entry_type: LSystemEntryType::Stem,
            thickness,
        });
    }
    pieces.push(LSystemEntry {
        pos: rotate_around(
            &rotate_around(
                &(*pos + (Vec3::Y * (length as f32))),
                pos,
                angle_z,
                &RotationDirection::Z,
            ),
            pos,
            angle_x,
            &RotationDirection::X,
        ),
        entry_type: LSystemEntryType::Branch {
            angle_x,
            angle_z,
            available_dirs: Directions::from_angles(angle_x, angle_z),
        },
        thickness,
    });
    pieces
}

impl Directions {
    fn from_angles(angle_x: f32, angle_z: f32) -> Self {
        let mut dirs = Directions::all();

        if angle_x > 0. {
            dirs.remove(Directions::NegXZ | Directions::NegXNegZ);
        } else if angle_x < 0. {
            dirs.remove(Directions::XZ | Directions::XNegZ);
        }

        if angle_z > 0. {
            dirs.remove(Directions::XNegZ | Directions::NegXNegZ);
        } else if angle_z < 0. {
            dirs.remove(Directions::XZ | Directions::NegXZ);
        }

        dirs
    }
}
