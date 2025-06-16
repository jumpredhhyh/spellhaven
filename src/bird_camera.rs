use crate::world_generation::chunk_loading::chunk_loader::ChunkLoader;
use bevy::app::App;
use bevy::core_pipeline::core_3d::Camera3d;
use bevy::ecs::component::Component;
use bevy::prelude::{Commands, Name, Plugin, Query, Startup, Transform, Update, Vec3, With};
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
        Camera3d::default(),
        Transform::from_xyz(-4.0, 6.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
        PanOrbitCamera::default(),
        //AtmosphereCamera::default(),
        Name::new("BirdCamera"),
    ));

    commands.spawn((
        Transform::from_xyz(0., 0., 0.),
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
    let Ok(mut camera_pivot) = camera_pivot.single_mut() else {
        return;
    };

    let Ok(camera) = camera.single() else {
        return;
    };
    camera_pivot.translation = camera.focus;
}
