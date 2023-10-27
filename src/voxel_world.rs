use std::collections::HashMap;
use std::sync::Arc;
use bevy::prelude::{Commands, Component, Resource, Transform, Vec2, Vec3};
use bevy_rapier3d::prelude::Collider;
use crate::chunk_generation::{CHUNK_SIZE, ChunkTaskData, VOXEL_SIZE};
use crate::chunk_loader::ChunkLoader;
use crate::generation_options::GenerationOptions;
use crate::mesh_generation::generate_mesh;
use crate::voxel_generation::generate_voxels;

pub const MAX_LOD: ChunkLod = ChunkLod::Eighth;

pub fn check_chunk_lod(position: &Vec3, chunk_pos: &[i32; 2], chunk_loader: &ChunkLoader) -> Option<ChunkLod> {
    let distance = Vec2::new(position.x / (CHUNK_SIZE[0] as f32 * VOXEL_SIZE), position.z / (CHUNK_SIZE[2] as f32 * VOXEL_SIZE)).distance(Vec2::new(chunk_pos[0] as f32, chunk_pos[1] as f32));

    for (index, range) in chunk_loader.lod_range.iter().enumerate() {
        if distance <= *range {
            return ChunkLod::from_u32(index as u32);
        }
    }

    None
}

fn chunk_pos_to_quad_tree_pos(chunk_pos: [i32; 3]) -> [i32; 2] {
    let divider = 2i32.pow(MAX_LOD as u32);
    [chunk_pos[0].div_floor(divider), chunk_pos[2].div_floor(divider)]
}

pub enum ChunkLod {
    Full = 0,
    Half = 1,
    Quarter = 2,
    Eighth = 3
}

impl ChunkLod {
    fn from_u32(number: u32) -> Option<Self> {
        match number {
            0 => {Some(Self::Full)}
            1 => {Some(Self::Half)}
            2 => {Some(Self::Quarter)}
            3 => {Some(Self::Eighth)}
            _ => {None}
        }
    }
}

pub struct QuadTreeVoxelWorld {
    chunk_trees: HashMap<[i32; 3], bool>
}

impl Default for QuadTreeVoxelWorld {
    fn default() -> Self {
        Self {
            chunk_trees: HashMap::default()
        }
    }
}

pub struct DefaultVoxelWorld {
    chunks: HashMap<[i32; 3], bool>
}

impl Default for DefaultVoxelWorld {
    fn default() -> Self {
        DefaultVoxelWorld {
            chunks: HashMap::new()
        }
    }
}

pub trait VoxelWorld {
    fn generate_chunk(chunk_position: [i32; 3], generation_options: Arc<GenerationOptions>) -> (Option<ChunkTaskData>, bool, [i32; 3]);
    fn has_chunk(&self, chunk_position: [i32; 3]) -> bool;
    fn add_chunk(&mut self, chunk_position: [i32; 3]) -> bool;
    fn remove_chunk(&mut self, chunk_position: [i32; 3]) -> bool;
}

impl Resource for DefaultVoxelWorld {}
impl Resource for QuadTreeVoxelWorld {}

impl VoxelWorld for DefaultVoxelWorld {
    fn generate_chunk(chunk_position: [i32; 3], generation_options: Arc<GenerationOptions>) -> (Option<ChunkTaskData>, bool, [i32; 3]) {
        let mesh = generate_mesh(generate_voxels(chunk_position, &generation_options));

        return (match mesh.0 {
            None => None,
            Some(mesh) => Some(ChunkTaskData{
                transform: Transform::from_xyz(chunk_position[0] as f32 * CHUNK_SIZE[0] as f32 * VOXEL_SIZE, -40.0, chunk_position[2] as f32 * CHUNK_SIZE[2] as f32 * VOXEL_SIZE),
                collider: Collider::trimesh(mesh.1, mesh.2),
                mesh: mesh.0
            })
        }, mesh.1, chunk_position.clone())
    }

    fn has_chunk(&self, chunk_position: [i32; 3]) -> bool {
        return self.chunks.contains_key(&chunk_position);
    }

    fn add_chunk(&mut self, chunk_position: [i32; 3]) -> bool {
        if self.has_chunk(chunk_position) {
            return false;
        }

        self.chunks.insert(chunk_position, true);
        true
    }

    fn remove_chunk(&mut self, chunk_position: [i32; 3]) -> bool {
        return match self.chunks.remove(&chunk_position) {
            None => { false }
            Some(_) => { true }
        }
    }
}

impl VoxelWorld for QuadTreeVoxelWorld {
    fn generate_chunk(chunk_position: [i32; 3], generation_options: Arc<GenerationOptions>) -> (Option<ChunkTaskData>, bool, [i32; 3]) {
        todo!()
    }

    fn has_chunk(&self, chunk_position: [i32; 3]) -> bool {
        todo!()
    }

    fn add_chunk(&mut self, chunk_position: [i32; 3]) -> bool {
        todo!()
    }

    fn remove_chunk(&mut self, chunk_position: [i32; 3]) -> bool {
        todo!()
    }
}