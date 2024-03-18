use crate::ui::fps_text::*;
use crate::ui::main_menu::MainMenuPlugin;
use crate::ui::task_text::{update_task_ui, ChunkTaskText, CountryTaskText};
use bevy::app::App;
use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::ecs::system::SystemId;
use bevy::prelude::*;

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            MainMenuPlugin::default(),
        ))
        .add_systems(Startup, register_spawn_ui_system)
        .add_systems(Update, (update_fps_ui, update_task_ui, update_triangle_ui));
    }
}

#[derive(Resource)]
pub struct UiSpawnCallback(pub SystemId);

fn register_spawn_ui_system(world: &mut World) {
    let id = world.register_system(spawn_ui);
    world.insert_resource(UiSpawnCallback(id));
}

fn spawn_ui(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                ..default()
            },
            ..default()
        })
        .with_children(|commands| {
            commands.spawn((
                TextBundle {
                    text: Text::from_section(
                        "FPS!",
                        TextStyle {
                            font_size: 32.0,
                            ..default()
                        },
                    ),
                    style: Style {
                        width: Val::Auto,
                        height: Val::Px(32.0),
                        margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(15.0), Val::Px(0.0)),
                        ..default()
                    },
                    ..default()
                },
                FpsText,
            ));
            commands.spawn((
                TextBundle {
                    text: Text::from_section(
                        "TRIANGLES!",
                        TextStyle {
                            font_size: 32.0,
                            ..default()
                        },
                    ),
                    style: Style {
                        width: Val::Auto,
                        height: Val::Px(32.0),
                        margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(15.0), Val::Px(0.0)),
                        ..default()
                    },
                    ..default()
                },
                TriangleText,
            ));
            commands.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Country Tasks!",
                        TextStyle {
                            font_size: 32.0,
                            ..default()
                        },
                    ),
                    style: Style {
                        width: Val::Auto,
                        height: Val::Px(32.0),
                        margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(15.0), Val::Px(0.0)),
                        ..default()
                    },
                    ..default()
                },
                CountryTaskText,
            ));
            commands.spawn((
                TextBundle {
                    text: Text::from_section(
                        "Chunk Tasks!",
                        TextStyle {
                            font_size: 32.0,
                            ..default()
                        },
                    ),
                    style: Style {
                        width: Val::Auto,
                        height: Val::Px(32.0),
                        margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(15.0), Val::Px(0.0)),
                        ..default()
                    },
                    ..default()
                },
                ChunkTaskText,
            ));
        });
}
