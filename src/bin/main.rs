use std::f32::consts::PI;
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::prelude::*;
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_rapier3d::prelude::{NoUserData, RapierDebugRenderPlugin, RapierPhysicsPlugin};
use bevy_screen_diags::ScreenDiagsTextPlugin;
use spellhaven::chunk_generation::ChunkGenerationPlugin;
use spellhaven::animations::AnimationPlugin;
use spellhaven::bird_camera::BirdCameraPlugin;
use spellhaven::player::PlayerPlugin;
use spellhaven::voxel_world::ChunkLod;

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
            BirdCameraPlugin,
            WorldInspectorPlugin::new()
        ))
        .add_systems(Startup, setup)
        .insert_resource(WireframeConfig {
            global: false,
        })
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        DirectionalLightBundle {
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
        },
        Name::new("Light")
    ));

    commands.insert_resource(AmbientLight {
        color: Color::ANTIQUE_WHITE,
        brightness: 0.05,
    });

    // commands.spawn(SceneBundle {
    //     scene: asset_server.load("player.gltf#Scene0"),
    //     transform: Transform::from_xyz(0., 150., 0.),
    //     ..default()
    // });
}