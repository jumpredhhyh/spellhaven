use bevy::prelude::*;
use bevy_atmosphere::prelude::AtmosphereCamera;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_rapier3d::prelude::{CharacterAutostep, CharacterLength, Collider, KinematicCharacterController, KinematicCharacterControllerOutput, RigidBody};
use crate::chunk_generation::{VOXEL_SIZE};
use crate::chunk_loader::ChunkLoader;

pub const STEP_HEIGHT: f32 = 1. * VOXEL_SIZE;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, setup)
            .add_systems(Update, (movement, move_camera, move_body));
    }
}

#[derive(Component)]
struct Player{
    velocity: Vec3,
    jumped: bool
}

#[derive(Component)]
struct PlayerBody;

#[derive(Component)]
struct PlayerCamera;

#[derive(Component)]
struct PlayerSteppingCastX;

#[derive(Component)]
struct PlayerSteppingCastNegX;

#[derive(Component)]
struct PlayerSteppingCastZ;

#[derive(Component)]
struct PlayerSteppingCastNegZ;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Player
    commands.spawn((
        RigidBody::KinematicPositionBased,
        TransformBundle::from_transform(Transform::from_xyz(0., 200., 0.)),
        Collider::cuboid(0.4, 0.9, 0.4),
        KinematicCharacterController {
            offset: CharacterLength::Absolute(0.01),
            autostep: Some(CharacterAutostep {
                min_width: CharacterLength::Absolute(0.01),
                max_height: CharacterLength::Absolute(VOXEL_SIZE + 0.1),
                include_dynamic_bodies: true
            }),
            ..default()
        },
        Player{velocity: Vec3::ZERO, jumped: false},
        ChunkLoader{
            unload_range: 20,
            load_range: 15
        },
        Name::new("Player")
    ));

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-4.0, 6.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
        AtmosphereCamera::default(),
        PlayerCamera,
        Name::new("PlayerCamera")
    ));

    commands.spawn((
        PbrBundle::default(),
        PlayerBody,
        Name::new("PlayerBody")
    )).with_children(|commands| {
        commands.spawn((
            SceneBundle {
                scene: asset_server.load("player.gltf#Scene0"),
                transform: Transform::from_xyz(0., 0.15, 0.),
                ..default()
            },
            Name::new("PlayerHead")
        ));
        commands.spawn((
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Capsule {
                    radius: 0.4,
                    depth: 0.3,
                    ..default()
                })),
                transform: Transform::from_xyz(0., -0.35, 0.),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                ..default()
            },
            Name::new("PlayerTorso")
        ));
    });
}

fn move_body(
    player: Query<&Transform, (With<Player>, Without<PlayerBody>)>,
    mut player_body: Query<&mut Transform, (With<PlayerBody>, Without<Player>)>
) {
    let difference = player.single().translation - player_body.single().translation;
    player_body.single_mut().translation += difference * 0.25;
}

fn move_camera(
    player: Query<&Transform, (With<Player>, Without<PlayerCamera>)>,
    mut camera: Query<&mut PanOrbitCamera, (With<PlayerCamera>, Without<Player>)>
) {
    let camera_position = camera.single().target_focus;
    let difference = (player.single().translation + Vec3::Y) - camera_position;
    camera.single_mut().target_focus += difference * 0.25;
}

fn movement(
    keyboard_input: Res<Input<KeyCode>>,
    time: Res<Time>,
    mut players: Query<(&mut KinematicCharacterController, &mut Player, Option<&KinematicCharacterControllerOutput>)>,
    player_camera: Query<&PanOrbitCamera, With<PlayerCamera>>
) {
    for (mut controller, mut player, controller_output) in &mut players {
        let mut move_direction = Vec3::ZERO;
        let mut last_movement = player.velocity;

        if player.jumped && controller_output.is_some() && controller_output.unwrap().grounded {
            player.jumped = false;
        }

        last_movement.x *= 0.8;
        last_movement.y *= 0.98;
        last_movement.z *= 0.8;

        // Directional movement
        if keyboard_input.pressed(KeyCode::W) || keyboard_input.pressed(KeyCode::Up) {
            move_direction.z -= 1.;
        }
        if keyboard_input.pressed(KeyCode::A) || keyboard_input.pressed(KeyCode::Left) {
            move_direction.x -= 1.;
        }
        if keyboard_input.pressed(KeyCode::S) || keyboard_input.pressed(KeyCode::Down) {
            move_direction.z += 1.;
        }
        if keyboard_input.pressed(KeyCode::D) || keyboard_input.pressed(KeyCode::Right) {
            move_direction.x += 1.;
        }

        let movement_speed = if keyboard_input.pressed(KeyCode::ShiftLeft) { 2. } else { 1. };

        // Rotate vector to camera
        move_direction = Quat::from_rotation_y(player_camera.single().alpha.unwrap_or(0.)).mul_vec3(move_direction.normalize_or_zero() * movement_speed);

        if controller_output.is_some() && !controller_output.unwrap().grounded {
            move_direction.y -= 0.4;
        }

        move_direction *= time.delta_seconds();

        // Jump if space pressed and the player is close enough to the ground
        if keyboard_input.pressed(KeyCode::Space) && controller_output.is_some() && controller_output.unwrap().grounded && !player.jumped {
            move_direction.y = 0.1;
            player.jumped = true;
        }

        let movement = move_direction + last_movement;
        controller.translation = Some(movement);
        player.velocity = movement;
    }
}

// fn kinematic_collision(
//     mut collision_event_reader: EventReader<Collision>,
//     mut bodies: Query<&RigidBody, Without<Player>>,
//     mut player_bodies: Query<(&mut Position, &ShapeHits), With<Player>>,
//     player_shape_hits_x: Query<&ShapeHits, With<PlayerSteppingCastX>>,
//     player_shape_hits_neg_x: Query<&ShapeHits, With<PlayerSteppingCastNegX>>,
//     player_shape_hits_z: Query<&ShapeHits, With<PlayerSteppingCastZ>>,
//     player_shape_hits_neg_z: Query<&ShapeHits, With<PlayerSteppingCastNegZ>>,
// ) {
//     // Iterate through collisions and move the kinematic body to resolve penetration
//     for Collision(contact) in collision_event_reader.iter() {
//         if let Ok((player_position, is_grounded)) = player_bodies.get_mut(contact.entity1) {
//             if let Ok(other_rb) = bodies.get_mut(contact.entity2) {
//                 handle_collision(player_position, player_shape_hits_x.single(), player_shape_hits_neg_x.single(), player_shape_hits_z.single(), player_shape_hits_neg_z.single(), other_rb, contact, false, !is_grounded.is_empty());
//             }
//         } else if let Ok((player_position, is_grounded)) = player_bodies.get_mut(contact.entity2) {
//             if let Ok(other_rb) = bodies.get_mut(contact.entity1) {
//                 handle_collision(player_position, player_shape_hits_x.single(), player_shape_hits_neg_x.single(), player_shape_hits_z.single(), player_shape_hits_neg_z.single(), other_rb, contact, true, !is_grounded.is_empty());
//             }
//         }
//     }
// }

// fn handle_collision(mut player_position: Mut<Position>, player_stepping_x: &ShapeHits, player_stepping_neg_x: &ShapeHits, player_stepping_z: &ShapeHits, player_stepping_neg_z: &ShapeHits, other_rb: &RigidBody, contact: &Contact, inverse: bool, is_grounded: bool) {
//     if contact.penetration <= Scalar::EPSILON || other_rb.is_kinematic() {
//         return;
//     }
//
//     let normal_to_use = if inverse { contact.normal * -1. } else { contact.normal };
//
//     if normal_to_use.y.abs() < 0.1 && is_grounded {
//         let corresponding_shape_hits: Option<&ShapeHits>;
//
//         if normal_to_use == Vec3::X {
//             corresponding_shape_hits = Some(player_stepping_x);
//         } else if normal_to_use == Vec3::NEG_X {
//             corresponding_shape_hits = Some(player_stepping_neg_x);
//         } else if normal_to_use == Vec3::Z {
//             corresponding_shape_hits = Some(player_stepping_z);
//         } else if normal_to_use == Vec3::NEG_Z {
//             corresponding_shape_hits = Some(player_stepping_neg_z);
//         } else {
//             corresponding_shape_hits = None;
//         }
//
//         if corresponding_shape_hits.is_some() {
//             if !corresponding_shape_hits.unwrap().is_empty() {
//                 let hit = corresponding_shape_hits.unwrap().as_slice().first().unwrap();
//                 if hit.time_of_impact > 0. {
//                     player_position.0.y += (STEP_HEIGHT + VOXEL_SIZE / 2.) - hit.time_of_impact;
//                     return;
//                 }
//             }
//         }
//     }
//
//     player_position.0 -= normal_to_use * contact.penetration;
// }