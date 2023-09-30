use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, TaskPool, TaskPoolBuilder};
use bevy_rapier3d::prelude::{Collider, RigidBody};
use futures_lite::future;
use vox_format::VoxData;
use crate::animations::SpawnAnimation;
use crate::chunk_loader::ChunkLoaderPlugin;
use crate::voxel_generation::vox_data_to_blocks;
use crate::voxel_world::{DefaultVoxelWorld, VoxelWorld};

pub const LEVEL_OF_DETAIL: i32 = 1;
pub const CHUNK_SIZE: [usize; 3] = [64, 64, 64];
pub const VOXEL_SIZE: f32 = 0.5 * LEVEL_OF_DETAIL as f32;

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
    Sand,
    Gray(u8),
    Custom(u8, u8, u8)
}

impl BlockType {
    pub fn get_color(&self) -> [f32; 4] {
        match self {
            BlockType::Air => [0., 0., 0., 0.],
            BlockType::Stone => [150. / 255., 160. / 255., 155. / 255., 1.],
            BlockType::Grass => [55. / 255., 195. /255., 95. / 255., 1.],
            BlockType::Gray(value) => [*value as f32 / 255., *value as f32 / 255., *value as f32 / 255., 1.],
            BlockType::Sand => [225. / 255., 195. / 255., 90. / 255., 1.],
            BlockType::Custom(r, g, b) => [*r as f32 / 255., *g as f32 / 255., *b as f32 / 255., 1.]
        }
    }
}

pub struct ChunkGenerationPlugin;

pub struct TreeModel {
    pub model: Vec<Vec<Vec<BlockType>>>
}

impl Resource for TreeModel {}

pub struct ChunkTaskPool(pub TaskPool);

impl Resource for ChunkTaskPool {}

impl Plugin for ChunkGenerationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ChunkLoaderPlugin)
            .add_systems(Startup, setup)
            .add_systems(Update, set_generated_chunks)
            .insert_resource(DefaultVoxelWorld::default())
            .insert_resource(TreeModel {
                model: vox_data_to_blocks(vox_format::from_file("assets/tree.vox").unwrap())
            })
            .insert_resource(ChunkTaskPool(TaskPoolBuilder::new().stack_size(3_000_000).build()));
    }
}

fn setup(
    mut commands: Commands,
    mut voxel_world: ResMut<DefaultVoxelWorld>,
    tree_model: Res<TreeModel>,
    task_pool: Res<ChunkTaskPool>
) {
    let size = 5;

    for x in -size..size + 1 {
        for z in -size..size + 1 {
            if !voxel_world.add_chunk([x, 0, z]) {
                continue;
            }

            let model = tree_model.model.clone();

            let task = task_pool.0.spawn(async move {
                DefaultVoxelWorld::generate_chunk([x, 0, z], &model)
            });

            commands.spawn(ChunkGenerationTask(task));
        }
    }
}

#[derive(Component)]
pub struct ChunkGenerationTask(pub Task<(Option<ChunkTaskData>, bool, [i32; 3])>);

#[derive(Component)]
pub struct Chunk(pub [i32; 3]);

fn set_generated_chunks(
    mut commands: Commands,
    mut chunks: Query<(Entity, &mut ChunkGenerationTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut voxel_world: ResMut<DefaultVoxelWorld>,
    tree_model: Res<TreeModel>,
    task_pool: Res<ChunkTaskPool>
){
    for (entity, mut task) in &mut chunks {
        if let Some(chunk_task_data_option) = future::block_on(future::poll_once(&mut task.0)) {
            if chunk_task_data_option.1 {
                let new_chunk_pos: [i32; 3] = [chunk_task_data_option.2[0], chunk_task_data_option.2[1] + 1, chunk_task_data_option.2[2]];

                if voxel_world.add_chunk(new_chunk_pos) {
                    let model = tree_model.model.clone();

                    let task = task_pool.0.spawn(async move {
                        DefaultVoxelWorld::generate_chunk(new_chunk_pos, &model)
                    });

                    commands.spawn(ChunkGenerationTask(task));
                }
            }

            if chunk_task_data_option.0.is_some() {
                let chunk_task_data = chunk_task_data_option.0.unwrap();

                commands.entity(entity).remove::<ChunkGenerationTask>();

                commands.entity(entity).insert((
                    RigidBody::Fixed,
                    chunk_task_data.collider,
                    PbrBundle {
                        mesh: meshes.add(chunk_task_data.mesh),
                        material: materials.add(Color::rgb(1., 1., 1.).into()),
                        transform: chunk_task_data.transform,
                        ..default()
                    },
                    Chunk(chunk_task_data_option.2),
                    SpawnAnimation::default()
                ));
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}