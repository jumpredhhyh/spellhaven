use std::ops::Range;

use bevy::math::{IVec3, Vec3};
use rand::Rng;

use crate::{
    utils::{rotate_around, vec_round_to_int, RotationDirection},
    world_generation::chunk_generation::{voxel_types::VoxelData, BlockType},
};

pub struct TreeLSystem {
    voxels: Vec<(IVec3, BlockType)>,
}

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
    pub fn grow_new(mut start_state: Vec<LSystemEntry>) -> Self {
        for _ in 0..3 {
            TreeLSystem::recurse_l_system(&mut start_state);
        }
        TreeLSystem::add_leafs(&mut start_state);

        let voxels: Vec<(IVec3, BlockType)> = start_state
            .iter()
            .flat_map(|entry| {
                let center = vec_round_to_int(&entry.pos);
                let thickness = entry.thickness.ceil() as i32;
                let mut blocks = vec![];

                for x in -thickness..thickness {
                    for y in -thickness..thickness {
                        for z in -thickness..thickness {
                            let current_pos_i = center + IVec3 { x, y, z };
                            let current_pos = current_pos_i.as_vec3();

                            if current_pos.distance(entry.pos) < entry.thickness {
                                blocks.push((
                                    current_pos_i,
                                    match entry.entry_type {
                                        LSystemEntryType::Leaf => BlockType::Grass(50),
                                        _ => BlockType::Path,
                                    },
                                ));
                            }
                        }
                    }
                }

                blocks
            })
            .collect();

        Self { voxels }
    }

    pub fn apply_tree_at(&self, insert_pos: IVec3, voxel_data: &mut VoxelData) {
        self.voxels
            .iter()
            .for_each(|voxel| voxel_data.set_block(voxel.0 + insert_pos, voxel.1));
    }

    fn recurse_l_system(data: &mut Vec<LSystemEntry>) {
        let mut i = 0usize;
        while i < data.len() {
            let entry = &data[i];
            match entry.entry_type {
                LSystemEntryType::Branch {
                    angle_x,
                    angle_z,
                    available_dirs: _,
                } => {
                    let mut rng = rand::rng();
                    let random_range: Range<f32> = -45.0..45.0;
                    let new_thickness = (entry.thickness - 0.5).max(0.75);

                    let branches: Vec<LSystemEntry> = create_branch_piece(
                        &entry.pos,
                        angle_x + rng.random_range(random_range.clone()),
                        angle_z + rng.random_range(random_range.clone()),
                        new_thickness,
                    )
                    .into_iter()
                    .chain(create_branch_piece(
                        &entry.pos,
                        angle_x + rng.random_range(random_range.clone()),
                        angle_z + rng.random_range(random_range.clone()),
                        new_thickness,
                    ))
                    .chain(create_branch_piece(
                        &entry.pos,
                        angle_x + rng.random_range(random_range.clone()),
                        angle_z + rng.random_range(random_range.clone()),
                        new_thickness,
                    ))
                    .chain(create_branch_piece(
                        &entry.pos,
                        angle_x + rng.random_range(random_range.clone()),
                        angle_z + rng.random_range(random_range.clone()),
                        new_thickness,
                    ))
                    .chain(create_branch_piece(
                        &entry.pos,
                        angle_x + rng.random_range(random_range.clone()),
                        angle_z + rng.random_range(random_range.clone()),
                        new_thickness,
                    ))
                    .chain(create_branch_piece(
                        &entry.pos,
                        angle_x + rng.random_range(random_range.clone()),
                        angle_z + rng.random_range(random_range),
                        new_thickness,
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
) -> Vec<LSystemEntry> {
    let mut rng = rand::rng();
    let length: usize = rng.random_range(7..11);

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
