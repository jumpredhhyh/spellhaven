use bevy::log::warn;
use bevy::prelude::{App, Commands, Component, Entity, Or, Plugin, Query, Res, ResMut, Transform, Update, Vec3, With};
use crate::animations::DespawnAnimation;
use crate::chunk_generation::{Chunk, CHUNK_SIZE, ChunkGenerationTask, ChunkTaskPool, VOXEL_SIZE};
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
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
    task_pool: Res<ChunkTaskPool>
) {
    for (chunk_loader, transform) in &chunk_loaders {
        let loader_chunk_pos = get_chunk_position(transform.translation);

        for x in -chunk_loader.load_range..chunk_loader.load_range + 1 {
            for z in -chunk_loader.load_range..chunk_loader.load_range + 1 {
                let chunk_position = [loader_chunk_pos[0] + x, 0, loader_chunk_pos[1] + z];

                if !voxel_world.add_chunk(chunk_position) {
                    continue;
                }

                let task = task_pool.0.spawn(async move {
                    DefaultVoxelWorld::generate_chunk(chunk_position)
                });


                commands.spawn(ChunkGenerationTask(task, chunk_position));
            }
        }
    }
}

fn unload_chunks(
    mut voxel_world: ResMut<DefaultVoxelWorld>,
    mut commands: Commands,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
    chunks: Query<(Entity, Option<&Chunk>, Option<&ChunkGenerationTask>), Or<(With<Chunk>, With<ChunkGenerationTask>)>>
) {
    for (entity, chunk_option, chunk_task_option) in &chunks {
        let mut should_unload = true;

        let chunk_position = match chunk_option {
            None => {
                match chunk_task_option {
                    None => {None}
                    Some(chunk_task) => {Some(chunk_task.1)}
                }
            }
            Some(chunk) => {Some(chunk.0)}
        };

        match chunk_position {
            None => {
                warn!("Shouldn't Happen!")
            }
            Some(chunk_position) => {
                for (chunk_loader, chunk_loader_transform) in &chunk_loaders {
                    let loader_chunk_pos = get_chunk_position(chunk_loader_transform.translation);
                    if (chunk_position[0] - loader_chunk_pos[0]).abs() < chunk_loader.unload_range && (chunk_position[2] - loader_chunk_pos[1]).abs() < chunk_loader.unload_range {
                        should_unload = false;
                        break;
                    }
                }

                if !should_unload {
                    continue;
                }

                if voxel_world.remove_chunk(chunk_position) {
                    match chunk_option {
                        None => {}
                        Some(_) => {
                            commands.entity(entity).remove::<Chunk>().insert(DespawnAnimation::default());
                        }
                    }
                    match chunk_task_option {
                        None => {}
                        Some(_) => {
                            commands.entity(entity).despawn();
                        }
                    }
                }
            }
        }
    }
}

fn get_chunk_position(global_position: Vec3) -> [i32; 2] {
    [(global_position.x / (CHUNK_SIZE[0] as f32 * VOXEL_SIZE)).floor() as i32, (global_position.z / (CHUNK_SIZE[2] as f32 * VOXEL_SIZE)).floor() as i32]
}