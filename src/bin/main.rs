use std::f32::consts::PI;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_panorbit_camera::{PanOrbitCamera, PanOrbitCameraPlugin};
use bevy_screen_diags::ScreenDiagsTextPlugin;
use bevy_xpbd_3d::prelude::PhysicsPlugins;
use spellhaven::chunk_generation::ChunkGenerationPlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PanOrbitCameraPlugin,
            ChunkGenerationPlugin,
            AtmospherePlugin,
            PhysicsPlugins::default(),
            ScreenDiagsTextPlugin,
            // PlayerPlugin,
            WireframePlugin
        ))
        .add_systems(Startup, setup)
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
            rotation: Quat::from_rotation_x(-PI / 4.),
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
}