use bevy::math::{IVec3, Vec3};

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
}

pub struct LSystemEntry {
    pub pos: Vec3,
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

        let voxels: Vec<(IVec3, BlockType)> = start_state
            .iter()
            .map(|entry| (vec_round_to_int(&entry.pos), BlockType::Path))
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
                    available_dirs,
                } => {
                    let branch_l = if available_dirs.contains(Directions::NegXZ) {
                        create_branch_piece(&entry.pos, angle_x - 15., angle_z + 15.)
                    } else {
                        vec![]
                    };
                    let branch_r = if available_dirs.contains(Directions::XNegZ) {
                        create_branch_piece(&entry.pos, angle_x + 15., angle_z - 15.)
                    } else {
                        vec![]
                    };
                    let branch_b = if available_dirs.contains(Directions::NegXNegZ) {
                        create_branch_piece(&entry.pos, angle_x - 15., angle_z - 15.)
                    } else {
                        vec![]
                    };
                    let branch_f = if available_dirs.contains(Directions::XZ) {
                        create_branch_piece(&entry.pos, angle_x + 15., angle_z + 15.)
                    } else {
                        vec![]
                    };
                    let branch_s = create_branch_piece(&entry.pos, angle_x, angle_z);
                    let branches: Vec<LSystemEntry> = branch_l
                        .into_iter()
                        .chain(branch_r.into_iter())
                        .chain(branch_b.into_iter())
                        .chain(branch_f.into_iter())
                        .chain(branch_s.into_iter())
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
}

fn create_branch_piece(pos: &Vec3, angle_x: f32, angle_z: f32) -> Vec<LSystemEntry> {
    const LEN: usize = 10;

    let mut pieces = Vec::new();

    for i in 0..LEN {
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
        });
    }
    pieces.push(LSystemEntry {
        pos: rotate_around(
            &rotate_around(
                &(*pos + (Vec3::Y * (LEN as f32))),
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
