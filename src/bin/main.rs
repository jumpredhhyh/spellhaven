use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::pbr::ExtendedMaterial;
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_atmosphere::prelude::AtmospherePlugin;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_panorbit_camera::PanOrbitCameraPlugin;
use bevy_rapier3d::prelude::{NoUserData, RapierPhysicsPlugin};
use spellhaven::animations::AnimationPlugin;
use spellhaven::debug_tools::debug_resource::SpellhavenDebugPlugin;
use spellhaven::player::PlayerPlugin;
use spellhaven::terrain_material::TerrainMaterial;
use spellhaven::ui::ui::GameUiPlugin;
use spellhaven::world_generation::chunk_generation::ChunkGenerationPlugin;
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
            ChunkGenerationPlugin,
            AtmospherePlugin,
            RapierPhysicsPlugin::<NoUserData>::default(),
            //RapierDebugRenderPlugin::default(),
            PlayerPlugin,
            WireframePlugin,
            AnimationPlugin,
            //BirdCameraPlugin,
            WorldInspectorPlugin::new(),
            GameUiPlugin,
            SpellhavenDebugPlugin,
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, TerrainMaterial>>::default(),
        ))
        .add_systems(Startup, setup)
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::RED,
        })
        .run();
}

fn setup(mut commands: Commands, _asset_server: Res<AssetServer>) {
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
        Name::new("Light"),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 50f32,
    });

    // commands.spawn(SceneBundle {
    //     scene: asset_server.load("player.gltf#Scene0"),
    //     transform: Transform::from_xyz(0., 150., 0.),
    //     ..default()
    // });
}
