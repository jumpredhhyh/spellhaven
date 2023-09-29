use std::f32::consts::PI;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use bevy_screen_diags::ScreenDiagsTextPlugin;
use spellhaven::chunk_generation::ChunkGenerationPlugin;
use spellhaven::chunk_loader::ChunkLoader;
use spellhaven::player::PlayerPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            ChunkGenerationPlugin,
            AtmospherePlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            //RapierDebugRenderPlugin::default(),
            ScreenDiagsTextPlugin,
            //PlayerPlugin,
            WireframePlugin
        ))
        .add_systems(Startup, setup)
        .add_systems(Update, move_camera_pivot)
        .insert_resource(WireframeConfig {
            global: false,
        })
        .run();
}

fn setup(
    mut commands: Commands,
) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 1000.,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 3.),
            ..default()
        },
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::ANTIQUE_WHITE,
        brightness: 0.05,
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-4.0, 6.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
        //AtmosphereCamera::default()
    ));

    commands.spawn((
        TransformBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        ChunkLoader {
            load_range: 25,
            unload_range: 30
        },
        CameraPivotPoint
    ));
}

#[derive(Component)]
struct CameraPivotPoint;

fn move_camera_pivot(
    camera: Query<&PanOrbitCamera>,
    mut camera_pivot: Query<&mut Transform, With<CameraPivotPoint>>
) {
    camera_pivot.single_mut().translation = camera.single().focus;
}