use std::collections::HashMap;
use std::sync::Arc;
use bevy::prelude::{Resource, Transform};
use bevy_rapier3d::prelude::Collider;
use crate::chunk_generation::{CHUNK_SIZE, ChunkTaskData, VOXEL_SIZE};
use crate::generation_options::{GenerationAssets, GenerationOptions};
use crate::mesh_generation::generate_mesh;
use crate::voxel_generation::generate_voxels;

pub struct DefaultVoxelWorld {
    chunks: HashMap<[i32; 3], bool>
}

impl DefaultVoxelWorld {
    pub fn default() -> Self {
        DefaultVoxelWorld {
            chunks: HashMap::new()
        }
    }
}

pub trait VoxelWorld {
    fn generate_chunk(chunk_position: [i32; 3], generation_assets: Arc<GenerationAssets>) -> (Option<ChunkTaskData>, bool, [i32; 3]);
    fn has_chunk(&self, chunk_position: [i32; 3]) -> bool;
    fn add_chunk(&mut self, chunk_position: [i32; 3]) -> bool;
    fn remove_chunk(&mut self, chunk_position: [i32; 3]) -> bool;
}

impl Resource for DefaultVoxelWorld {}

impl VoxelWorld for DefaultVoxelWorld {
    fn generate_chunk(chunk_position: [i32; 3], generation_assets: Arc<GenerationAssets>) -> (Option<ChunkTaskData>, bool, [i32; 3]) {
        let mesh = generate_mesh(generate_voxels(chunk_position, &GenerationOptions::get_options(generation_assets)));

        return (match mesh.0 {
            None => None,
            Some(mesh) => Some(ChunkTaskData{
                transform: Transform::from_xyz(chunk_position[0] as f32 * CHUNK_SIZE[0] as f32 * VOXEL_SIZE, -40.0, chunk_position[2] as f32 * CHUNK_SIZE[2] as f32 * VOXEL_SIZE),
                collider: Collider::trimesh(mesh.1, mesh.2),
                mesh: mesh.0
            })
        }, mesh.1, chunk_position)
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