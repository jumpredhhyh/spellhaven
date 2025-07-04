use crate::world_generation::chunk_loading::chunk_loader::ChunkLoader;
use bevy::app::App;
use bevy::prelude::{
    default, Camera3dBundle, Commands, Component, Name, Plugin, Query, Startup, Transform,
    TransformBundle, Update, Vec3, With,
};
use bevy_panorbit_camera::PanOrbitCamera;

pub struct BirdCameraPlugin;

impl Plugin for BirdCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, move_camera_pivot);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-4.0, 6.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
        //AtmosphereCamera::default(),
        Name::new("BirdCamera"),
    ));

    commands.spawn((
        TransformBundle::from_transform(Transform::from_xyz(0., 0., 0.)),
        ChunkLoader::default(),
        CameraPivotPoint,
        Name::new("BirdCameraPivot"),
    ));
}

#[derive(Component)]
struct CameraPivotPoint;

fn move_camera_pivot(
    camera: Query<&PanOrbitCamera>,
    mut camera_pivot: Query<&mut Transform, With<CameraPivotPoint>>,
) {
    camera_pivot.single_mut().translation = camera.single().focus;
}
