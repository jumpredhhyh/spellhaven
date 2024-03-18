use bevy::{
    pbr::MaterialExtension,
    prelude::*,
    render::{
        mesh::MeshVertexAttribute,
        render_resource::{AsBindGroup, ShaderRef, VertexFormat},
    },
};

use crate::world_generation::chunk_generation::voxel_types::{Vec4, VoxelArray, VoxelPalette};

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct TerrainMaterial {
    #[uniform(100)]
    pub palette: VoxelPalette,
    #[uniform(100)]
    pub chunk_blocks: VoxelArray,
    #[uniform(100)]
    pub chunk_pos: IVec3,
    #[uniform(100)]
    pub chunk_lod: i32,
    #[uniform(100)]
    pub min_chunk_height: i32,
}

impl MaterialExtension for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain_material.wgsl".into()
    }
}
