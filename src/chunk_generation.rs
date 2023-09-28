use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use bevy_xpbd_3d::prelude::{Collider, CollisionLayers, PhysicsLayer, Position, RigidBody};
use futures_lite::future;
use crate::voxel_world::{DefaultVoxelWorld, VoxelWorld};

pub const LEVEL_OF_DETAIL: i32 = 1;
pub const CHUNK_SIZE: [usize; 3] = [32, 256, 32];
pub const VOXEL_SIZE: f32 = 0.25 * LEVEL_OF_DETAIL as f32;

pub struct ChunkTaskData{
    pub mesh: Mesh,
    pub transform: Transform,
    pub collider: Collider
}

#[derive(Copy, Clone, PartialEq)]
pub enum BlockType {
    Air,
    Stone,
    Grass,
    Gray(u8)
}

#[derive(PhysicsLayer)]
pub enum CollisionLayer {
    Player,
    Ground
}

pub struct ChunkGenerationPlugin;

impl Plugin for ChunkGenerationPlugin {
    fn build(&self, app: &mut App) {
        app
            //.add_plugins(ChunkLoaderPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, set_generated_chunks)
            .insert_resource(DefaultVoxelWorld::default());
    }
}

fn setup(
    mut commands: Commands,
    mut voxel_world: ResMut<DefaultVoxelWorld>
) {
    let thread_pool = AsyncComputeTaskPool::get();

    let size = 40;

    for x in -size..size + 1 {
        for z in -size..size + 1 {
            if !voxel_world.add_chunk([x, z]) {
                continue;
            }

            let task = thread_pool.spawn(async move {
                DefaultVoxelWorld::generate_chunk([x, z])
            });

            commands.spawn(ChunkGenerationTask(task));
        }
    }
}

#[derive(Component)]
pub struct ChunkGenerationTask(pub Task<Option<ChunkTaskData>>);

#[derive(Component)]
pub struct Chunk;

fn set_generated_chunks(
    mut commands: Commands,
    mut chunks: Query<(Entity, &mut ChunkGenerationTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
){
    for (entity, mut task) in &mut chunks {
        if let Some(chunk_task_data_option) = future::block_on(future::poll_once(&mut task.0)) {
            if chunk_task_data_option.is_some() {
                let chunk_task_data = chunk_task_data_option.unwrap();

                commands.entity(entity).remove::<ChunkGenerationTask>();

                commands.entity(entity).insert((
                    RigidBody::Static,
                    //chunk_task_data.collider,
                    PbrBundle {
                        mesh: meshes.add(chunk_task_data.mesh),
                        material: materials.add(Color::rgb(1., 1., 1.).into()),
                        transform: chunk_task_data.transform,
                        ..default()
                    },
                    Position(chunk_task_data.transform.translation),
                    CollisionLayers::all_masks::<CollisionLayer>().add_group(CollisionLayer::Ground),
                    Chunk
                ));
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}