use bevy::{pbr::wireframe::WireframeConfig, prelude::*, window::PresentMode};
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use spellhaven::wave_function_collapse::WaveFunctionCollapsePlugin;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Spellhaven".into(),
                        present_mode: PresentMode::Immediate,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
            // PanOrbitCameraPlugin,
            // BirdCameraPlugin,
            WorldInspectorPlugin::new(),
            WaveFunctionCollapsePlugin,
        ))
        .add_systems(Startup, setup)
        .insert_resource(WireframeConfig {
            global: false,
            default_color: Color::srgb(1., 0., 0.),
        })
        .run();
}

fn setup(mut _commands: Commands, _asset_server: Res<AssetServer>) {
    // commands.spawn((
    //     DirectionalLightBundle {
    //         directional_light: DirectionalLight {
    //             shadows_enabled: true,
    //             illuminance: 1000.,
    //             ..default()
    //         },
    //         transform: Transform {
    //             translation: Vec3::new(0.0, 2.0, 0.0),
    //             rotation: Quat::from_rotation_x(-PI / 3.),
    //             ..default()
    //         },
    //         ..default()
    //     },
    //     Name::new("Light"),
    // ));

    // commands.insert_resource(AmbientLight {
    //     color: Color::WHITE,
    //     brightness: 50f32,
    // });

    // commands.spawn(SceneBundle {
    //     scene: asset_server.load("player.gltf#Scene0"),
    //     transform: Transform::from_xyz(0., 150., 0.),
    //     ..default()
    // });
}
