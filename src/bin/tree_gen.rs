use bevy::core_pipeline::experimental::taa::TemporalAntiAliasing;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::pbr::{ExtendedMaterial, ScreenSpaceAmbientOcclusion};
use bevy::prelude::*;
use bevy::render::camera::Exposure;
use bevy::window::PresentMode;
use bevy_atmosphere::plugin::AtmosphereCamera;
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use rand::Rng;
use spellhaven::animations::AnimationPlugin;
use spellhaven::debug_tools::debug_resource::SpellhavenDebugPlugin;
use spellhaven::terrain_material::TerrainMaterial;
use spellhaven::world_generation::chunk_generation::mesh_generation::generate_mesh;
use spellhaven::world_generation::chunk_generation::voxel_types::VoxelData;
use spellhaven::world_generation::chunk_generation::BlockType;
use spellhaven::world_generation::foliage_generation::tree_l_system::{
    Directions, LSystemEntry, LSystemEntryType, TreeLSystem,
};
use spellhaven::world_generation::voxel_world::ChunkLod;
use std::f32::consts::PI;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Spellhaven".into(),
                    present_mode: PresentMode::Immediate,
                    ..default()
                }),
                ..default()
            }),
            PanOrbitCameraPlugin,
            AtmospherePlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            //RapierDebugRenderPlugin::default(),
            WireframePlugin,
            AnimationPlugin,
            //BirdCameraPlugin,
            WorldInspectorPlugin::new(),
            SpellhavenDebugPlugin,
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, TerrainMaterial>>::default(),
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, rebuild_tree_system)
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::srgb(1., 0., 0.),
        })
        .run();
}

#[derive(Component)]
struct TreeGen;

fn setup(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, TerrainMaterial>>>,
) {
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            illuminance: 1000.,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 3.),
            ..default()
        },
        Name::new("Light"),
    ));

    commands.spawn((
        Camera3d::default(),
        Msaa::Off,
        TemporalAntiAliasing::default(),
        Transform::from_xyz(-4.0, 6.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            far: 2f32.powi(20),
            ..default()
        }),
        Exposure { ev100: 10f32 },
        PanOrbitCamera::default(),
        AtmosphereCamera::default(),
        Name::new("CAMMIE"),
        ScreenSpaceAmbientOcclusion::default(),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 50f32,
    });

    let mut blocks = VoxelData::default();
    blocks.set_block(IVec3::new(1, 1, 1), BlockType::Grass(0));

    let l_system = TreeLSystem::grow_new(create_branch_piece(&Vec3::ZERO));
    l_system.apply_tree_at(IVec3::new(31, 0, 31), &mut blocks);

    let mesh = generate_mesh(&blocks, 0, ChunkLod::Full);

    commands.spawn((
        Mesh3d(meshes.add(mesh.expect("No Mesh").0)),
        MeshMaterial3d(materials.add(ExtendedMaterial {
            base: Color::WHITE.into(),
            extension: TerrainMaterial {
                chunk_blocks: blocks.array,
                palette: blocks.palette,
                chunk_pos: IVec3::ZERO,
                chunk_lod: ChunkLod::Full.multiplier_i32(),
                min_chunk_height: 0,
            },
        })),
        TreeGen,
    ));
}

fn create_branch_piece(pos: &Vec3) -> Vec<LSystemEntry> {
    const LEN: usize = 10;

    let mut pieces = Vec::new();
    let thickness: f32 = 2.0;

    let mut rng = rand::rng();
    let pos_offset = Vec3 {
        x: rng.random(),
        y: 0.,
        z: rng.random(),
    };

    for i in 1..LEN {
        pieces.push(LSystemEntry {
            pos: pos + pos_offset + Vec3::Y * i as f32,
            entry_type: LSystemEntryType::Stem,
            thickness,
        });
    }
    pieces.push(LSystemEntry {
        pos: pos + pos_offset + Vec3::Y * LEN as f32,
        entry_type: LSystemEntryType::Branch {
            angle_x: 0.,
            angle_z: 0.,
            available_dirs: Directions::all(),
        },
        thickness,
    });
    pieces
}

fn rebuild_tree_system(
    mut tree_entities: Query<
        (
            &mut Mesh3d,
            &mut MeshMaterial3d<ExtendedMaterial<StandardMaterial, TerrainMaterial>>,
        ),
        With<TreeGen>,
    >,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, TerrainMaterial>>>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if !input.pressed(KeyCode::Space) {
        return;
    }

    for (mut tree_mesh, mut tree_material) in &mut tree_entities {
        let mut blocks = VoxelData::default();
        blocks.set_block(IVec3::new(1, 1, 1), BlockType::Grass(0));

        let l_system = TreeLSystem::grow_new(create_branch_piece(&Vec3::ZERO));
        l_system.apply_tree_at(IVec3::new(31, 1, 31), &mut blocks);

        let new_mesh = generate_mesh(&blocks, 0, ChunkLod::Full);

        meshes.remove(tree_mesh.id());
        tree_mesh.0 = meshes.add(new_mesh.expect("No Mesh").0);

        materials.remove(tree_material.0.id());
        tree_material.0 = materials.add(ExtendedMaterial {
            base: Color::WHITE.into(),
            extension: TerrainMaterial {
                chunk_blocks: blocks.array,
                palette: blocks.palette,
                chunk_pos: IVec3::ZERO,
                chunk_lod: ChunkLod::Full.multiplier_i32(),
                min_chunk_height: 0,
            },
        });
    }
}
