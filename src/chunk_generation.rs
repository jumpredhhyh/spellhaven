use std::collections::HashMap;
use std::sync::Arc;
use bevy::prelude::*;
use bevy::tasks::{Task, TaskPool, TaskPoolBuilder};
use bevy_rapier3d::prelude::{Collider, RigidBody};
use futures_lite::future;
use crate::chunk_loader::{ChunkLoader, ChunkLoaderPlugin, get_chunk_position};
use crate::generation_options::GenerationOptionsResource;
use crate::quad_tree_data::QuadTreeNode;
use crate::quad_tree_data::QuadTreeNode::{Data, Node};
use crate::voxel_world::{ChunkGenerationResult, ChunkLod, MAX_LOD, QuadTreeVoxelWorld, VoxelWorld};

//pub const LEVEL_OF_DETAIL: i32 = 1;
pub const CHUNK_SIZE: [usize; 3] = [64, 64, 64];
pub const VOXEL_SIZE: f32 = 0.5;

pub struct ChunkTaskData{
    pub mesh: Mesh,
    pub transform: Transform,
    pub collider: Option<Collider>
}

#[derive(Copy, Clone, PartialEq)]
pub enum BlockType {
    Air,
    Stone,
    Grass,
    Sand,
    Path,
    Gray(u8),
    Custom(u8, u8, u8),
    StructureDebug(u8, u8, u8),
}

impl BlockType {
    pub fn get_color(&self) -> [f32; 4] {
        match self {
            BlockType::Air => [0., 0., 0., 0.],
            BlockType::Stone => [150. / 255., 160. / 255., 155. / 255., 1.],
            BlockType::Grass => [55. / 255., 195. /255., 95. / 255., 1.],
            BlockType::Gray(value) => [*value as f32 / 255., *value as f32 / 255., *value as f32 / 255., 1.],
            BlockType::Sand => [225. / 255., 195. / 255., 90. / 255., 1.],
            BlockType::Custom(r, g, b) => [*r as f32 / 255., *g as f32 / 255., *b as f32 / 255., 1.],
            BlockType::StructureDebug(r, g, b) => [*r as f32 / 255., *g as f32 / 255., *b as f32 / 255., 1.],
            BlockType::Path => [100. / 255., 65. / 255., 50. / 255., 1.]
        }
    }
}

pub struct ChunkGenerationPlugin;

pub struct ChunkTaskPool(pub TaskPool);

impl Resource for ChunkTaskPool {}

impl Plugin for ChunkGenerationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ChunkLoaderPlugin)
            //.add_systems(Startup, setup)
            .add_systems(Update, set_generated_chunks)
            .add_systems(Update, start_generating_quadtree_chunks.after(upgrade_quad_trees))
            .add_systems(Update, upgrade_quad_trees.after(set_generated_chunks))
            .insert_resource(QuadTreeVoxelWorld::default())
            .insert_resource(ChunkTaskPool(TaskPoolBuilder::new().num_threads(2).stack_size(3_000_000).build()))
            .insert_resource(GenerationOptionsResource::default());
    }
}

// fn setup(
//     mut commands: Commands,
//     mut voxel_world: ResMut<QuadTreeVoxelWorld>,
// ) {
//     let size = 1;
//
//     for x in -size..size + 1 {
//         for z in -size..size + 1 {
//             //commands.spawn(ChunkGenerator([x, z]));
//         }
//     }
// }

#[derive(Component)]
pub struct ChunkGenerationTask(pub Task<ChunkGenerationResult>, pub [i32; 3], pub Entity);

#[derive(Component)]
pub struct Chunk(pub [i32; 3]);

#[derive(Component, Reflect)]
pub struct ChunkParent(pub [i32; 2]);

#[derive(Component)]
pub struct ChunkGenerator(pub [i32; 2]);

fn start_generating_quadtree_chunks(
    mut commands: Commands,
    mut voxel_world: ResMut<QuadTreeVoxelWorld>,
    chunk_generators: Query<(Entity, &ChunkGenerator)>,
    generation_options: Res<GenerationOptionsResource>,
    task_pool: Res<ChunkTaskPool>,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
) {
    for (entity, chunk_generator) in &chunk_generators {
        match voxel_world.get_chunk(chunk_generator.0) {
            None => {}
            Some(chunk_tree) => {
                let tree = generate_quad_tree_chunk(entity, MAX_LOD, [0, 0], chunk_generator.0, &chunk_loaders, &task_pool.0, &generation_options, &mut commands);

                **chunk_tree = Some(tree);

                commands.entity(entity).insert(
                    Name::new("Chunk [".to_owned() + &chunk_generator.0[0].to_string() + ", " + &chunk_generator.0[1].to_string() + "]")
                );

                commands.entity(entity).remove::<ChunkGenerator>();
            }
        }
    }
}

fn generate_quad_tree_chunk(owner: Entity, current_lod: ChunkLod, current_lod_pos: [i32; 2], owner_chunk_pos: [i32; 2], chunk_loaders: &Query<(&ChunkLoader, &Transform)>, task_pool: &TaskPool, generation_options: &Res<GenerationOptionsResource>, commands: &mut Commands) -> QuadTreeNode<HashMap<i32, Entity>> {
    let mut divide = false;

    if current_lod != ChunkLod::Full {
        for (chunk_loader, transform) in chunk_loaders {
            let loader_chunk_position = get_chunk_position(transform.translation, current_lod);
            let current_chunk_pos = [owner_chunk_pos[0] * current_lod.inverse_multiplier_i32() + current_lod_pos[0], owner_chunk_pos[1] * current_lod.inverse_multiplier_i32() + current_lod_pos[1]];
            let position_difference = [loader_chunk_position[0] - current_chunk_pos[0], loader_chunk_position[1] - current_chunk_pos[1]];
            let current_range = chunk_loader.lod_range[MAX_LOD.usize() - current_lod.usize()];
            if position_difference[0].abs() <= current_range && position_difference[1].abs() <= current_range {
                divide = true;
            }
        }
    }

    if divide {
        return Node(
            Box::new(generate_quad_tree_chunk(owner, current_lod.previous(), [current_lod_pos[0] * 2, current_lod_pos[1] * 2], owner_chunk_pos, chunk_loaders, task_pool, generation_options, commands)),
            Box::new(generate_quad_tree_chunk(owner, current_lod.previous(), [current_lod_pos[0] * 2 + 1, current_lod_pos[1] * 2], owner_chunk_pos, chunk_loaders, task_pool, generation_options, commands)),
            Box::new(generate_quad_tree_chunk(owner, current_lod.previous(), [current_lod_pos[0] * 2, current_lod_pos[1] * 2 + 1], owner_chunk_pos, chunk_loaders, task_pool, generation_options, commands)),
            Box::new(generate_quad_tree_chunk(owner, current_lod.previous(), [current_lod_pos[0] * 2 + 1, current_lod_pos[1] * 2 + 1], owner_chunk_pos, chunk_loaders, task_pool, generation_options, commands))
        );
    }

    let generation_options = Arc::clone(&generation_options.0);

    let task = task_pool.spawn(async move {
        QuadTreeVoxelWorld::generate_chunk(owner_chunk_pos, current_lod, current_lod_pos, generation_options, 0)
    });

    let child = commands.spawn((
        ChunkGenerationTask(task, [owner_chunk_pos[0], 0, owner_chunk_pos[1]], owner),
        Name::new(format!("SubChunk[lod: {current_lod:?}, pos:{current_lod_pos:?}]")),
        Visibility::Visible
    )).id();

    commands.entity(owner).insert(SpatialBundle::default());

    commands.entity(owner).add_child(child);

    let mut map = HashMap::new();
    map.insert(0, child);

    Data(map)
}

pub(crate) fn upgrade_quad_trees(
    mut commands: Commands,
    generation_options: Res<GenerationOptionsResource>,
    task_pool: Res<ChunkTaskPool>,
    mut voxel_world: ResMut<QuadTreeVoxelWorld>,
    chunks: Query<(Entity, &ChunkParent)>,
    chunk_loaders: Query<(&ChunkLoader, &Transform)>,
) {
    for chunk in &chunks {
        let boxed_tree = voxel_world.get_chunk(chunk.1.0).expect("Chunk not found!");

        match *boxed_tree.clone() {
            None => {}
            Some(chunk_tree) => {
                let tree = upgrade_tree_recursion(chunk.0, chunk_tree, MAX_LOD, [0, 0], chunk.1.0, &chunk_loaders, &task_pool.0, &generation_options, &mut commands);

                **boxed_tree = Some(tree);
            }
        }
    }
}

fn upgrade_tree_recursion(owner: Entity, current_node: QuadTreeNode<HashMap<i32, Entity>>, current_lod: ChunkLod, current_lod_pos: [i32; 2], owner_chunk_pos: [i32; 2], chunk_loaders: &Query<(&ChunkLoader, &Transform)>, task_pool: &TaskPool, generation_options: &Res<GenerationOptionsResource>, commands: &mut Commands) -> QuadTreeNode<HashMap<i32, Entity>> {
    match current_node {
        Data(children) => {
            let mut divide = false;

            if current_lod != ChunkLod::Full {
                for (chunk_loader, transform) in chunk_loaders {
                    let loader_chunk_position = get_chunk_position(transform.translation, current_lod);
                    let current_chunk_pos = [owner_chunk_pos[0] * current_lod.inverse_multiplier_i32() + current_lod_pos[0], owner_chunk_pos[1] * current_lod.inverse_multiplier_i32() + current_lod_pos[1]];
                    let position_difference = [loader_chunk_position[0] - current_chunk_pos[0], loader_chunk_position[1] - current_chunk_pos[1]];
                    let current_range = chunk_loader.lod_range[MAX_LOD.usize() - current_lod.usize()];
                    if position_difference[0].abs() <= current_range && position_difference[1].abs() <= current_range {
                        divide = true;
                    }
                }
            }

            if !divide {
                return Data(children);
            }

            for (_, child) in children {
                match commands.get_entity(child) {
                    None => {}
                    Some(mut child) => {
                        child.despawn();
                    }
                }
            }

            Node(
                Box::new(generate_quad_tree_chunk(owner, current_lod.previous(), [current_lod_pos[0] * 2, current_lod_pos[1] * 2], owner_chunk_pos, chunk_loaders, task_pool, generation_options, commands)),
                Box::new(generate_quad_tree_chunk(owner, current_lod.previous(), [current_lod_pos[0] * 2 + 1, current_lod_pos[1] * 2], owner_chunk_pos, chunk_loaders, task_pool, generation_options, commands)),
                Box::new(generate_quad_tree_chunk(owner, current_lod.previous(), [current_lod_pos[0] * 2, current_lod_pos[1] * 2 + 1], owner_chunk_pos, chunk_loaders, task_pool, generation_options, commands)),
                Box::new(generate_quad_tree_chunk(owner, current_lod.previous(), [current_lod_pos[0] * 2 + 1, current_lod_pos[1] * 2 + 1], owner_chunk_pos, chunk_loaders, task_pool, generation_options, commands))
            )
        }
        Node(a, b, c, d) => {
            let mut divide = false;

            if current_lod != ChunkLod::Full {
                for (chunk_loader, transform) in chunk_loaders {
                    let loader_chunk_position = get_chunk_position(transform.translation, current_lod);
                    let current_chunk_pos = [owner_chunk_pos[0] * current_lod.inverse_multiplier_i32() + current_lod_pos[0], owner_chunk_pos[1] * current_lod.inverse_multiplier_i32() + current_lod_pos[1]];
                    let position_difference = [loader_chunk_position[0] - current_chunk_pos[0], loader_chunk_position[1] - current_chunk_pos[1]];
                    let current_range = chunk_loader.lod_range[MAX_LOD.usize() - current_lod.usize()];
                    if position_difference[0].abs() <= current_range && position_difference[1].abs() <= current_range {
                        divide = true;
                    }
                }
            }

            if divide {
                return Node(
                    Box::new(upgrade_tree_recursion(owner, *a, current_lod.previous(), [current_lod_pos[0] * 2, current_lod_pos[1] * 2], owner_chunk_pos, chunk_loaders, task_pool, generation_options, commands)),
                    Box::new(upgrade_tree_recursion(owner, *b, current_lod.previous(), [current_lod_pos[0] * 2 + 1, current_lod_pos[1] * 2], owner_chunk_pos, chunk_loaders, task_pool, generation_options, commands)),
                    Box::new(upgrade_tree_recursion(owner, *c, current_lod.previous(), [current_lod_pos[0] * 2, current_lod_pos[1] * 2 + 1], owner_chunk_pos, chunk_loaders, task_pool, generation_options, commands)),
                    Box::new(upgrade_tree_recursion(owner, *d, current_lod.previous(), [current_lod_pos[0] * 2 + 1, current_lod_pos[1] * 2 + 1], owner_chunk_pos, chunk_loaders, task_pool, generation_options, commands))
                );
            }

            remove_recursive(*a, commands);
            remove_recursive(*b, commands);
            remove_recursive(*c, commands);
            remove_recursive(*d, commands);

            generate_quad_tree_chunk(owner, current_lod, current_lod_pos, owner_chunk_pos, chunk_loaders, task_pool, generation_options, commands)
        }
    }
}

fn remove_recursive(current_node: QuadTreeNode<HashMap<i32, Entity>>, commands: &mut Commands) {
    match current_node {
        Data(entities) => {
            for (_, entity) in entities {
                match commands.get_entity(entity) {
                    None => {}
                    Some(mut entity) => {
                        entity.despawn();
                    }
                }
            }
        }
        Node(a, b, c, d) => {
            remove_recursive(*a, commands);
            remove_recursive(*b, commands);
            remove_recursive(*c, commands);
            remove_recursive(*d, commands);
        }
    }
}

fn set_generated_chunks(
    mut commands: Commands,
    mut chunks: Query<(Entity, &mut ChunkGenerationTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    task_pool: Res<ChunkTaskPool>,
    generation_options: Res<GenerationOptionsResource>,
    mut voxel_world: ResMut<QuadTreeVoxelWorld>,
){
    for (entity, mut task) in &mut chunks {
        if let Some(chunk_task_data_option) = future::block_on(future::poll_once(&mut task.0)) {
            if chunk_task_data_option.generate_above {
                match voxel_world.get_chunk(chunk_task_data_option.parent_pos) {
                    None => {info!("Owner not found!")}
                    Some(tree) => {
                        match tree.as_mut() {
                            None => {info!("Owner not found!")}
                            Some(ref mut tree) => {
                                match tree.get_data(<ChunkLod as Into<i32>>::into(MAX_LOD) - <ChunkLod as Into<i32>>::into(chunk_task_data_option.lod), chunk_task_data_option.lod_position) {
                                    None => {info!("Map not found! depth: {0}, pos: [{1}, {2}]", <ChunkLod as Into<i32>>::into(MAX_LOD) - <ChunkLod as Into<i32>>::into(chunk_task_data_option.lod), chunk_task_data_option.lod_position[0], chunk_task_data_option.lod_position[1])}
                                    Some(map) => {
                                        let new_height = chunk_task_data_option.chunk_height + 1;
                                        let generation_options = generation_options.0.clone();
                                        let child_task = task_pool.0.spawn(async move {
                                            QuadTreeVoxelWorld::generate_chunk(chunk_task_data_option.parent_pos, chunk_task_data_option.lod, chunk_task_data_option.lod_position, generation_options, new_height)
                                        });

                                        let child = commands.spawn((
                                            ChunkGenerationTask(child_task, [chunk_task_data_option.parent_pos[0], 0, chunk_task_data_option.parent_pos[1]], task.2),
                                            Name::new(format!("SubChunk[lod: {0:?}, pos: {1:?}, height: {new_height}]", chunk_task_data_option.lod, chunk_task_data_option.lod_position)),
                                            Visibility::Visible
                                        )).id();

                                        commands.entity(task.2).add_child(child);

                                        map.insert(new_height, child);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if chunk_task_data_option.task_data.is_some() {
                let chunk_task_data = chunk_task_data_option.task_data.unwrap();

                commands
                    .entity(entity)
                    .remove::<ChunkGenerationTask>()
                    .insert((
                        PbrBundle {
                            mesh: meshes.add(chunk_task_data.mesh),
                            material: materials.add(Color::rgb(1., 1., 1.).into()),
                            transform: chunk_task_data.transform,
                            ..default()
                        },
                        Chunk([chunk_task_data_option.parent_pos[0], chunk_task_data_option.chunk_height, chunk_task_data_option.parent_pos[1]]),
                        //SpawnAnimation::default()
                    ));

                if chunk_task_data_option.lod == ChunkLod::Full {
                    commands.entity(entity).insert((
                        RigidBody::Fixed,
                        chunk_task_data.collider.unwrap(),
                    ));
                }
            } else {
                commands.entity(entity).despawn();
            }
        }
    }
}