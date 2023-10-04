use std::f32::consts::PI;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use bevy_screen_diags::ScreenDiagsTextPlugin;
use spellhaven::chunk_generation::ChunkGenerationPlugin;
use spellhaven::animations::AnimationPlugin;
use spellhaven::bird_camera::BirdCameraPlugin;

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
            WireframePlugin,
            AnimationPlugin,
            BirdCameraPlugin
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
            rotation: Quat::from_rotation_x(-PI / 3.),
            ..default()
        },
        ..default()
    });

    commands.insert_resource(AmbientLight {
        color: Color::ANTIQUE_WHITE,
        brightness: 0.05,
    });
}