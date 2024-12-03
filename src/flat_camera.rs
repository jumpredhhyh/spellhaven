use bevy::{
    app::{Plugin, Startup, Update},
    core_pipeline::core_2d::{Camera2d, Camera2dBundle},
    ecs::{
        component::Component,
        event::EventReader,
        query::With,
        system::{Commands, Query, Res},
    },
    input::{
        mouse::{MouseButton, MouseMotion, MouseScrollUnit, MouseWheel},
        ButtonInput,
    },
    math::Vec3,
    render::camera::{self, OrthographicProjection},
    transform::components::Transform,
    utils::info,
};

pub struct FlatCameraPlugin;

impl Plugin for FlatCameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, move_camera);
    }
}

#[derive(Component)]
pub struct FlatCamera;

fn setup(mut commands: Commands) {
    commands.spawn((Camera2dBundle::default(), FlatCamera));
}

fn move_camera(
    buttons: Res<ButtonInput<MouseButton>>,
    mut scroll_evr: EventReader<MouseWheel>,
    mut motion_evr: EventReader<MouseMotion>,
    mut cameras: Query<(&mut Transform, &mut OrthographicProjection), With<FlatCamera>>,
) {
    if buttons.pressed(MouseButton::Right) {
        for ev in motion_evr.read() {
            for (mut transform, projection) in &mut cameras {
                transform.translation +=
                    Vec3::new(-ev.delta.x, ev.delta.y, 0.) * projection.scale * 0.925;
            }
        }
    }

    for ev in scroll_evr.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                for (_, mut projection) in &mut cameras {
                    projection.scale = (projection.scale - ev.y * 0.1).max(0.1);
                }
            }
            MouseScrollUnit::Pixel => {
                println!(
                    "Scroll (pixel units): vertical: {}, horizontal: {}",
                    ev.y, ev.x
                );
            }
        }
    }
}
