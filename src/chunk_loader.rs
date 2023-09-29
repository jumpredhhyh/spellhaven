use bevy::prelude::{App, Commands, Component, Entity, Plugin, Query, ResMut, Transform, Update, Vec3, With};
use bevy::tasks::AsyncComputeTaskPool;
use crate::chunk_generation::{Chunk, CHUNK_SIZE, ChunkGenerationTask, VOXEL_SIZE};
use crate::voxel_world::{DefaultVoxelWorld, VoxelWorld};

pub struct ChunkLoaderPlugin;

impl Plugin for ChunkLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (load_chunks, unload_chunks));
    }
}

#[derive(Component)]
pub struct ChunkLoader{
    pub load_range: i32,
    pub unload_range: i32
}

fn load_chunks(
    mut voxel_world: ResMut<DefaultVoxelWorld>,
    mut commands: Commands,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>
) {
    let thread_pool = AsyncComputeTaskPool::get();

    for (chunk_loader, transform) in &chunk_loaders {
        let loader_chunk_pos = get_chunk_position(transform.translation);

        for x in -chunk_loader.load_range..chunk_loader.load_range + 1 {
            for z in -chunk_loader.load_range..chunk_loader.load_range + 1 {
                let chunk_position = [loader_chunk_pos[0] + x, loader_chunk_pos[1] + z];

                if !voxel_world.add_chunk(chunk_position) {
                    continue;
                }

                let task = thread_pool.spawn(async move {
                    DefaultVoxelWorld::generate_chunk(chunk_position)
                });


                commands.spawn(ChunkGenerationTask(task));
            }
        }
    }
}

fn unload_chunks(
    mut voxel_world: ResMut<DefaultVoxelWorld>,
    mut commands: Commands,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
    chunks: Query<(Entity, &Transform), With<Chunk>>
) {
    for (entity, transform) in &chunks {
        let chunk_position = get_chunk_position(transform.translation);

        let mut should_unload = true;

        for (chunk_loader, chunk_loader_transform) in &chunk_loaders {
            let loader_chunk_pos = get_chunk_position(chunk_loader_transform.translation);
            if (chunk_position[0] - loader_chunk_pos[0]).abs() < chunk_loader.unload_range && (chunk_position[1] - loader_chunk_pos[1]).abs() < chunk_loader.unload_range {
                should_unload = false;
                break;
            }
        }

        if !should_unload {
            continue;
        }

        if voxel_world.remove_chunk(chunk_position) {
            commands.entity(entity).despawn();
        }
    }
}

fn get_chunk_position(global_position: Vec3) -> [i32; 2] {
    [(global_position.x / (CHUNK_SIZE[0] as f32 * VOXEL_SIZE)).floor() as i32, (global_position.z / (CHUNK_SIZE[2] as f32 * VOXEL_SIZE)).floor() as i32]
}