use std::ops::Range;

use bevy::math::Vec3;
use rand::{rngs::StdRng, Rng};

use crate::{
    utils::{rotate_around, RotationDirection},
    world_generation::{
        chunk_generation::{BlockType, VOXEL_SIZE},
        foliage_generation::tree_l_system::{LSystem, LSystemEntry},
    },
};

pub struct PineLSystem;

#[derive(Clone, Copy)]
pub enum PineEntryType {
    Log,
    Stem { extensions: i32 },
    Branch { direction: Vec3, extensions: i32 },
    Leaf,
}

impl LSystem<PineEntryType> for PineLSystem {
    fn get_start_state(position: Vec3, rng: &mut StdRng) -> Vec<LSystemEntry<PineEntryType>> {
        let mut entries = vec![];

        let length = (rng.random_range(9.0..11.0) / VOXEL_SIZE) as usize;
        let mut last_length = length;

        entries.extend(Self::create_straight_piece_dir(
            position,
            Vec3::Y,
            2.0,
            length,
            PineEntryType::Log,
            PineEntryType::Stem { extensions: 1 },
        ));

        for i in 0..8 {
            let length = (rng.random_range(2.5..3.5) / VOXEL_SIZE) as usize;

            entries.extend(Self::create_straight_piece_dir(
                position + Vec3::Y * last_length as f32,
                Vec3::Y,
                2.0 - i as f32 * 0.1,
                length,
                PineEntryType::Log,
                PineEntryType::Stem { extensions: 2 },
            ));

            last_length += length;
        }

        entries
    }

    fn process_tree(mut start_state: &mut Vec<LSystemEntry<PineEntryType>>, rng: &mut StdRng) {
        for _ in 0..8 {
            Self::recurse_l_system(&mut start_state, rng);
        }
        Self::add_leafs(&mut start_state);
    }

    fn get_block_from_entry(entry: &LSystemEntry<PineEntryType>) -> BlockType {
        match entry.entry_type {
            PineEntryType::Leaf => BlockType::Grass(100),
            _ => BlockType::Path,
        }
    }

    fn recurse_entry(
        entry: &LSystemEntry<PineEntryType>,
        rng: &mut StdRng,
        branches: &mut Vec<LSystemEntry<PineEntryType>>,
    ) {
        match entry.entry_type {
            PineEntryType::Stem { extensions } => {
                let new_thickness = (entry.thickness - 0.5).max(0.75);

                for _ in 0..6 {
                    let turn_angle = rng.random_range(0.0..360.0);
                    let down_angle = rng.random_range(50.0..65.0);
                    let mut direction = Vec3::X;

                    direction =
                        rotate_around(&direction, &Vec3::ZERO, -down_angle, &RotationDirection::Z);

                    direction =
                        rotate_around(&direction, &Vec3::ZERO, turn_angle, &RotationDirection::Y);

                    branches.extend(Self::create_straight_piece_dir(
                        entry.pos,
                        direction,
                        new_thickness,
                        (rng.random_range(5.5..7.5) / VOXEL_SIZE) as usize,
                        PineEntryType::Log,
                        PineEntryType::Branch {
                            extensions,
                            direction,
                        },
                    ));
                }
            }
            PineEntryType::Branch {
                direction,
                extensions,
            } => {
                if extensions <= 0 {
                    return;
                }

                let new_extensions = extensions - 1;
                let new_thickness = (entry.thickness - 0.5).max(0.75);

                branches.extend(Self::create_straight_piece_dir(
                    entry.pos,
                    direction,
                    new_thickness,
                    (rng.random_range(2.5..4.0) / VOXEL_SIZE) as usize,
                    PineEntryType::Log,
                    PineEntryType::Branch {
                        extensions: new_extensions,
                        direction,
                    },
                ));

                branches.push(LSystemEntry {
                    pos: entry.pos,
                    thickness: 2.0,
                    entry_type: PineEntryType::Leaf,
                });

                // let random_range: Range<f32> = -5.0..5.0;
                // let new_thickness = (entry.thickness - 0.5).max(0.75);

                // for _ in 0..6 {
                //     branches.extend(Self::create_straight_piece(
                //         &entry.pos,
                //         angle_x + rng.random_range(random_range.clone()),
                //         angle_z + rng.random_range(random_range.clone()),
                //         new_thickness,
                //         (rng.random_range(3.5..5.5) / VOXEL_SIZE) as usize,
                //         PineEntryType::Log,
                //         PineEntryType::Branch { angle_x, angle_z },
                //     ));
                // }
            }
            _ => {}
        }
    }
}

impl PineLSystem {
    fn add_leafs(data: &mut Vec<LSystemEntry<PineEntryType>>) {
        let mut i = 0usize;
        while i < data.len() {
            let entry = &data[i];
            match entry.entry_type {
                PineEntryType::Branch {
                    direction: _,
                    extensions: _,
                } => {
                    let branches: Vec<LSystemEntry<PineEntryType>> = vec![LSystemEntry {
                        pos: entry.pos,
                        entry_type: PineEntryType::Leaf,
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
