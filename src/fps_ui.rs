use bevy::app::App;
use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

pub struct FpsUi;

impl Plugin for FpsUi {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(FrameTimeDiagnosticsPlugin::default())
            .add_systems(Startup, spawn_fps_ui)
            .add_systems(Update, update_fps_ui);
    }
}

#[derive(Component)]
pub struct FpsText;

fn spawn_fps_ui(mut commands: Commands) {
    commands
        .spawn(
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                ..default()
            }
        ).with_children(|commands| {
        commands.spawn((
            TextBundle {
                text: Text::from_section(
                    "FPS!",
                    TextStyle {
                        font_size: 32.0,
                        ..default()
                    }
                ),
                style: Style {
                    width: Val::Auto,
                    height: Val::Px(32.0),
                    margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(15.0), Val::Px(0.0)),
                    ..default()
                },
                ..default()
            },
            FpsText
        ));
    });
}

fn update_fps_ui(
    mut texts: Query<&mut Text, With<FpsText>>,
    diagnostics: Res<DiagnosticsStore>
) {
    for mut text in &mut texts {
        if let Some(fps) = diagnostics.get(FrameTimeDiagnosticsPlugin::FPS).and_then(|fps| fps.smoothed()) {
            text.sections[0].value = format!("FPS: {:?}", fps.floor());
        }
    }
}