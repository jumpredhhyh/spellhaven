use std::collections::HashMap;
use std::sync::Arc;
use bevy::prelude::{Resource, Transform, Vec2, Vec3};
use bevy_rapier3d::prelude::Collider;
use crate::chunk_generation::{CHUNK_SIZE, ChunkTaskData, VOXEL_SIZE};
use crate::chunk_loader::ChunkLoader;
use crate::generation_options::GenerationOptions;
use crate::mesh_generation::generate_mesh;
use crate::voxel_generation::generate_voxels;

pub const MAX_LOD: ChunkLod = ChunkLod::Sixtyfourth;

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
    let divider = 2i32.pow(MAX_LOD.u32());
    [chunk_pos[0].div_floor(divider), chunk_pos[2].div_floor(divider)]
}

#[derive(Copy, Clone)]
pub enum ChunkLod {
    Full = 1,
    Half = 2,
    Quarter = 3,
    Eighth = 4,
    Sixteenth = 5,
    Thirtytwoth = 6,
    Sixtyfourth = 7,
}

impl ChunkLod {
    pub const fn usize(self) -> usize { self as usize }
    pub const fn u32(self) -> u32 { self as u32 }
    pub const fn i32(self) -> i32 { self as i32 }
    pub const fn f32(self) -> f32 { self as u8 as f32 }
    pub const fn f64(self) -> f64 { self as u8 as f64 }
    pub const fn multiplier_i32(self) -> i32 { 2i32.pow(self as u32 - 1) }
    pub const fn multiplier_f32(self) -> f32 { self.multiplier_i32() as f32 }

    fn from_u32(number: u32) -> Option<Self> {
        match number {
            1 => {Some(Self::Full)}
            2 => {Some(Self::Half)}
            3 => {Some(Self::Quarter)}
            4 => {Some(Self::Eighth)}
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
        let mesh = generate_mesh(generate_voxels(chunk_position, &generation_options, ChunkLod::Full), ChunkLod::Full);

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
        let mesh = generate_mesh(generate_voxels(chunk_position, &generation_options, MAX_LOD), MAX_LOD);

        return (match mesh.0 {
            None => None,
            Some(mesh) => Some(ChunkTaskData{
                transform: Transform::from_xyz(chunk_position[0] as f32 * CHUNK_SIZE[0] as f32 * VOXEL_SIZE * MAX_LOD.multiplier_f32(), -40.0, chunk_position[2] as f32 * CHUNK_SIZE[2] as f32 * VOXEL_SIZE * MAX_LOD.multiplier_f32()),
                collider: Collider::trimesh(mesh.1, mesh.2),
                mesh: mesh.0
            })
        }, mesh.1, chunk_position.clone())
    }

    fn has_chunk(&self, chunk_position: [i32; 3]) -> bool {
        return self.chunk_trees.contains_key(&chunk_position);
    }

    fn add_chunk(&mut self, chunk_position: [i32; 3]) -> bool {
        if self.has_chunk(chunk_position) {
            return false;
        }

        self.chunk_trees.insert(chunk_position, true);
        true
    }

    fn remove_chunk(&mut self, chunk_position: [i32; 3]) -> bool {
        return match self.chunk_trees.remove(&chunk_position) {
            None => { false }
            Some(_) => { true }
        }
    }
}