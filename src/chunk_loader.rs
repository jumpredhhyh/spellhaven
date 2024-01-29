use bevy::log::info;
use bevy::prelude::{App, Commands, Component, Entity, IntoSystemConfigs, Plugin, Query, ResMut, Transform, Update, Vec3};
use crate::animations::DespawnAnimation;
use crate::chunk_generation::{CHUNK_SIZE, ChunkGenerationTask, ChunkGenerator, ChunkParent, VOXEL_SIZE};
use crate::voxel_world::{ChunkLod, MAX_LOD, QuadTreeVoxelWorld, VoxelWorld};

pub struct ChunkLoaderPlugin;

impl Plugin for ChunkLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (load_chunks, unload_chunks).after(crate::chunk_generation::upgrade_quad_trees));
    }
}

#[derive(Component)]
pub struct ChunkLoader{
    pub load_range: i32,
    pub unload_range: i32,
    pub lod_range: [i32; MAX_LOD.usize() - 1]
}

impl Default for ChunkLoader {
    fn default() -> Self {
        Self {
            load_range: 8,
            unload_range: 10,
            lod_range: [3, 3, 3, 3, 3, 3, 3],
        }
    }
}

fn load_chunks(
    mut voxel_world: ResMut<QuadTreeVoxelWorld>,
    mut commands: Commands,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
) {
    for (chunk_loader, transform) in &chunk_loaders {
        let loader_chunk_pos = get_chunk_position(transform.translation, MAX_LOD);

        for x in -chunk_loader.load_range..chunk_loader.load_range + 1 {
            for z in -chunk_loader.load_range..chunk_loader.load_range + 1 {
                let chunk_pos = [loader_chunk_pos[0] + x, loader_chunk_pos[1] + z];
                if !voxel_world.has_chunk(chunk_pos) {
                    commands.spawn((ChunkGenerator(chunk_pos), ChunkParent(chunk_pos)));
                    if !voxel_world.add_chunk(chunk_pos, None) {
                        info!("Chunk already exists!");
                    }
                }
            }
        }
    }
}

fn unload_chunks(
    mut voxel_world: ResMut<QuadTreeVoxelWorld>,
    mut commands: Commands,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
    chunks: Query<(Entity, &ChunkParent)>,
    children: Query<(Entity, &ChunkGenerationTask)>
) {
    for (entity, chunk_parent) in &chunks {
        let mut should_unload = true;

        let chunk_position = chunk_parent.0;

        for (chunk_loader, chunk_loader_transform) in &chunk_loaders {
            let loader_chunk_pos = get_chunk_position(chunk_loader_transform.translation, MAX_LOD);
            if (chunk_position[0] - loader_chunk_pos[0]).abs() < chunk_loader.unload_range && (chunk_position[1] - loader_chunk_pos[1]).abs() < chunk_loader.unload_range {
                should_unload = false;
                break;
            }
        }

        if !should_unload {
            continue;
        }

        if voxel_world.remove_chunk(chunk_position) {
            let mut chunk_owner = commands.entity(entity);
            chunk_owner.remove::<ChunkParent>().insert(DespawnAnimation::default());
            for child in &children {
                if child.1.1 == entity {
                    info!("Cancelled Child!");
                    commands.entity(child.0).remove::<ChunkGenerationTask>();
                }
            }
        }
    }
}

pub fn get_chunk_position(global_position: Vec3, lod: ChunkLod) -> [i32; 2] {
    [(global_position.x / (CHUNK_SIZE[0] as f32 * VOXEL_SIZE * lod.multiplier_f32())).floor() as i32, (global_position.z / (CHUNK_SIZE[2] as f32 * VOXEL_SIZE * lod.multiplier_f32())).floor() as i32]
}