use std::collections::HashMap;
use std::sync::Arc;
use bevy::prelude::{Entity, IVec2, Resource, Transform};
use bevy_rapier3d::prelude::Collider;
use crate::world_generation::chunk_generation::{CHUNK_SIZE, ChunkTaskData, VOXEL_SIZE};
use crate::world_generation::chunk_generation::mesh_generation::generate_mesh;
use crate::world_generation::chunk_generation::voxel_generation::generate_voxels;
use crate::world_generation::chunk_loading::country_cache::CountryCache;
use crate::world_generation::chunk_loading::quad_tree_data::QuadTreeNode;
use crate::world_generation::generation_options::GenerationOptions;

pub const MAX_LOD: ChunkLod = ChunkLod::OneTwentyEight;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ChunkLod {
    Full = 1,
    Half = 2,
    Quarter = 3,
    Eighth = 4,
    Sixteenth = 5,
    Thirtytwoth = 6,
    Sixtyfourth = 7,
    OneTwentyEight = 8,
    TwoFiftySix = 9,
}

impl From<ChunkLod> for i32 {
    fn from(value: ChunkLod) -> Self {
        value as Self
    }
}

impl ChunkLod {
    pub const fn usize(self) -> usize { self as usize }
    pub const fn u32(self) -> u32 { self as u32 }
    pub const fn i32(self) -> i32 { self as i32 }
    pub const fn f32(self) -> f32 { self as u8 as f32 }
    pub const fn f64(self) -> f64 { self as u8 as f64 }
    pub const fn multiplier_i32(self) -> i32 { 2i32.pow(self as u32 - 1) }
    pub const fn multiplier_f32(self) -> f32 { self.multiplier_i32() as f32 }
    pub const fn inverse_multiplier_i32(self) -> i32 { 2i32.pow(MAX_LOD as u32 - self as u32) }
    pub fn previous(self) -> Self {
        ChunkLod::from_u8(self as u8 - 1).expect("Mapping doesn't exist!")
    }

    fn from_u8(number: u8) -> Option<Self> {
        match number {
            1 => {Some(Self::Full)}
            2 => {Some(Self::Half)}
            3 => {Some(Self::Quarter)}
            4 => {Some(Self::Eighth)}
            5 => {Some(Self::Sixteenth)}
            6 => {Some(Self::Thirtytwoth)}
            7 => {Some(Self::Sixtyfourth)}
            8 => {Some(Self::OneTwentyEight)}
            9 => {Some(Self::TwoFiftySix)}
            _ => {None}
        }
    }
}

pub struct QuadTreeVoxelWorld {
    chunk_trees: HashMap<[i32; 2], Box<Option<QuadTreeNode<HashMap<i32, Entity>>>>>
}

impl Default for QuadTreeVoxelWorld {
    fn default() -> Self {

        Self {
            chunk_trees: HashMap::default()
        }
    }
}

pub trait VoxelWorld {
    fn generate_chunk(chunk_position: IVec2, chunk_lod: ChunkLod, lod_position: IVec2, generation_options: Arc<GenerationOptions>, chunk_height: i32, country_cache: &CountryCache) -> ChunkGenerationResult;
    fn has_chunk(&self, chunk_position: [i32; 2]) -> bool;
    fn add_chunk(&mut self, chunk_position: [i32; 2], chunk: Option<QuadTreeNode<HashMap<i32, Entity>>>) -> bool;
    fn remove_chunk(&mut self, chunk_position: [i32; 2]) -> bool;
    fn get_chunk(&mut self, chunk_position: [i32; 2]) -> Option<&mut Box<Option<QuadTreeNode<HashMap<i32, Entity>>>>>;
}

impl Resource for QuadTreeVoxelWorld {}

pub struct ChunkGenerationResult {
    pub task_data: Option<ChunkTaskData>,
    pub generate_above: bool,
    pub parent_pos: IVec2,
    pub lod: ChunkLod,
    pub lod_position: IVec2,
    pub chunk_height: i32
}

impl VoxelWorld for QuadTreeVoxelWorld {
    fn generate_chunk(parent_pos: IVec2, chunk_lod: ChunkLod, lod_position: IVec2, generation_options: Arc<GenerationOptions>, chunk_height: i32, country_cache: &CountryCache) -> ChunkGenerationResult {
        let new_chunk_pos = [parent_pos.x * MAX_LOD.multiplier_i32() + lod_position.x * chunk_lod.multiplier_i32(), chunk_height, parent_pos.y * MAX_LOD.multiplier_i32() + lod_position.y * chunk_lod.multiplier_i32()];
        let mesh = generate_mesh(generate_voxels(new_chunk_pos, &generation_options, chunk_lod, &country_cache), chunk_lod);

        return ChunkGenerationResult{
            task_data: match mesh.0 {
                None => None,
                Some(mesh) => Some(ChunkTaskData{
                    transform: Transform::from_xyz(new_chunk_pos[0] as f32 * CHUNK_SIZE[0] as f32 * VOXEL_SIZE, 0.0, new_chunk_pos[2] as f32 * CHUNK_SIZE[2] as f32 * VOXEL_SIZE),
                    collider: if chunk_lod == ChunkLod::Full { Some(Collider::trimesh(mesh.1, mesh.2)) } else { None },
                    mesh: mesh.0
                })},
            generate_above: mesh.1,
            parent_pos,
            lod: chunk_lod,
            lod_position,
            chunk_height,
        };
    }

    fn has_chunk(&self, chunk_position: [i32; 2]) -> bool {
        let map = self.chunk_trees.get(&chunk_position);
        map.is_some()
    }

    fn add_chunk(&mut self, chunk_position: [i32; 2], chunk: Option<QuadTreeNode<HashMap<i32, Entity>>>) -> bool {
        if self.has_chunk(chunk_position) {
            return false
        }

        self.chunk_trees.insert(chunk_position, Box::new(chunk));

        true
    }

    fn remove_chunk(&mut self, chunk_position: [i32; 2]) -> bool {
        self.chunk_trees.remove(&chunk_position).is_some()
    }

    fn get_chunk(&mut self, chunk_position: [i32; 2]) -> Option<&mut Box<Option<QuadTreeNode<HashMap<i32, Entity>>>>> {
        self.chunk_trees.get_mut(&chunk_position)
    }
}