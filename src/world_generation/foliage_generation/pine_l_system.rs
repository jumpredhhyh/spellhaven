use std::ops::Range;

use bevy::math::Vec3;
use rand::{rngs::StdRng, Rng};

use crate::world_generation::{chunk_generation::{BlockType, VOXEL_SIZE}, foliage_generation::tree_l_system::{LSystem, LSystemEntry}};

pub struct PineLSystem;


#[derive(Clone, Copy)]
pub enum PineEntryType {
    Stem,
    Branch {
        angle_x: f32,
        angle_z: f32,
    },
    Leaf,
}

impl LSystem<PineEntryType> for PineLSystem {
    fn get_start_state(position: Vec3, rng: &mut StdRng) -> Vec<LSystemEntry<PineEntryType>> {
        Self::create_straight_piece(
            &position,
            0.,
            0.,
            2.0,
            (rng.random_range(3.5..5.5) / VOXEL_SIZE) as usize,
            PineEntryType::Stem,
            PineEntryType::Branch { angle_x: 0., angle_z: 0. }
        )
    }
    
    fn process_tree(mut start_state: &mut Vec<LSystemEntry<PineEntryType>>, rng: &mut StdRng) {
        for _ in 0..3 {
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
    
    fn recurse_entry(entry: &LSystemEntry<PineEntryType>, rng: &mut StdRng, branches: &mut Vec<LSystemEntry<PineEntryType>>) {
        match entry.entry_type {
            PineEntryType::Branch {
                angle_x,
                angle_z,
            } => {
                let random_range: Range<f32> = -45.0..45.0;
                let new_thickness = (entry.thickness - 0.5).max(0.75);

                for _ in 0..6 {
                    branches.extend(
                        Self::create_straight_piece(
                            &entry.pos,
                            angle_x + rng.random_range(random_range.clone()),
                            angle_z + rng.random_range(random_range.clone()),
                            new_thickness,
                            (rng.random_range(3.5..5.5) / VOXEL_SIZE) as usize,
                            PineEntryType::Stem,
                            PineEntryType::Branch {
                                angle_x,
                                angle_z,
                            },
                        )
                    );
                }
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
                    angle_x: _,
                    angle_z: _,
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