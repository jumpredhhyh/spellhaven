use bevy::prelude::*;
use bevy_atmosphere::prelude::AtmosphereCamera;
use bevy_panorbit_camera::PanOrbitCamera;
use bevy_xpbd_3d::math::{Quaternion, Scalar, Vector};
use bevy_xpbd_3d::{PhysicsSchedule, PhysicsStepSet, SubstepSchedule, SubstepSet};
use bevy_xpbd_3d::prelude::{Collider, Collision, CollisionLayers, Contact, LinearVelocity, Position, RigidBody, ShapeCaster, ShapeHits, SpatialQueryFilter};
use crate::chunk_generation::{CollisionLayer, VOXEL_SIZE};
use crate::chunk_loader::ChunkLoader;

pub const STEP_HEIGHT: f32 = 1. * VOXEL_SIZE;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(PhysicsSchedule, movement.before(PhysicsStepSet::BroadPhase))
            .add_systems(
                // Run collision handling in substep schedule
                SubstepSchedule,
                kinematic_collision.in_set(SubstepSet::SolveUserConstraints),
            )
            .add_systems(Update, (move_camera, move_body));
    }
}

#[derive(Component)]
struct Player;

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
) {
    // Player
    commands.spawn((
        RigidBody::Kinematic,
        Transform::default(),
        Position(Vector::Y * 60.0),
        Collider::cuboid(0.7, 1.7, 0.7),
        CollisionLayers::all_masks::<CollisionLayer>().add_group(CollisionLayer::Player),
        // Cast the player shape downwards to detect when the player is grounded
        ShapeCaster::new(
            Collider::cuboid(0.65, 1.65, 0.65),
            Vector::ZERO,
            Quaternion::default(),
            Vector::NEG_Y,
        )
            .with_query_filter(SpatialQueryFilter::new().with_masks([CollisionLayer::Ground]))
            .with_max_time_of_impact(0.05)
            .with_max_hits(1),
        Player,
        ChunkLoader {
            load_range: 10,
            unload_range: 20
        }
    )).with_children(|parent| {
        parent.spawn((
            create_step_shape_caster(Vec3::X),
            PlayerSteppingCastX
        ));
        parent.spawn((
            create_step_shape_caster(Vec3::NEG_X),
            PlayerSteppingCastNegX
        ));
        parent.spawn((
            create_step_shape_caster(Vec3::Z),
            PlayerSteppingCastZ
        ));
        parent.spawn((
            create_step_shape_caster(Vec3::NEG_Z),
            PlayerSteppingCastNegZ
        ));
    });

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(-4.0, 6.5, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
        AtmosphereCamera::default(),
        PlayerCamera
    ));

    commands.spawn((
        PbrBundle {
           mesh: meshes.add(Mesh::from(shape::Capsule {
               radius: 0.4,
               ..default()
           })),
           material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
           ..default()
        },
        PlayerBody
    ));
}

fn create_step_shape_caster(direction: Vec3) -> ShapeCaster {
    ShapeCaster::new(
        Collider::cuboid(0.65, 1.7, 0.65),
        (direction * (VOXEL_SIZE / 2.)) + (Vector::Y * (STEP_HEIGHT + VOXEL_SIZE / 2.)),
        Quaternion::default(),
        Vector::NEG_Y,
    )
        .with_max_time_of_impact(STEP_HEIGHT)
        .with_query_filter(SpatialQueryFilter::new().with_masks([CollisionLayer::Ground]))
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
    let difference = player.single().translation - camera_position;
    camera.single_mut().target_focus += difference * 0.25;
}

fn movement(
    keyboard_input: Res<Input<KeyCode>>,
    mut players: Query<(&mut LinearVelocity, &ShapeHits), With<Player>>,
    player_camera: Query<&PanOrbitCamera, With<PlayerCamera>>
) {
    for (mut linear_velocity, ground_hits) in &mut players {
        let mut move_direction = Vec3::ZERO;

        // Reset vertical velocity if grounded, otherwise apply gravity
        if !ground_hits.is_empty() {
            linear_velocity.y = -0.1;
        } else {
            linear_velocity.y -= 0.4;
        }

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

        let movement_speed = if keyboard_input.pressed(KeyCode::ShiftLeft) { 3. } else { 1.5 };

        // Rotate vector to camera
        move_direction = Quaternion::from_rotation_y(player_camera.single().alpha.unwrap_or(0.)).mul_vec3(move_direction.normalize_or_zero() * movement_speed);

        // Jump if space pressed and the player is close enough to the ground
        if keyboard_input.pressed(KeyCode::Space) && !ground_hits.is_empty() {
            move_direction.y += 10.0;
        }

        linear_velocity.0 += move_direction;

        // Slow player down
        linear_velocity.x *= 0.8;
        linear_velocity.y *= 0.98;
        linear_velocity.z *= 0.8;
    }
}

fn kinematic_collision(
    mut collision_event_reader: EventReader<Collision>,
    mut bodies: Query<&RigidBody, Without<Player>>,
    mut player_bodies: Query<(&mut Position, &ShapeHits), With<Player>>,
    player_shape_hits_x: Query<&ShapeHits, With<PlayerSteppingCastX>>,
    player_shape_hits_neg_x: Query<&ShapeHits, With<PlayerSteppingCastNegX>>,
    player_shape_hits_z: Query<&ShapeHits, With<PlayerSteppingCastZ>>,
    player_shape_hits_neg_z: Query<&ShapeHits, With<PlayerSteppingCastNegZ>>,
) {
    // Iterate through collisions and move the kinematic body to resolve penetration
    for Collision(contact) in collision_event_reader.iter() {
        if let Ok((player_position, is_grounded)) = player_bodies.get_mut(contact.entity1) {
            if let Ok(other_rb) = bodies.get_mut(contact.entity2) {
                handle_collision(player_position, player_shape_hits_x.single(), player_shape_hits_neg_x.single(), player_shape_hits_z.single(), player_shape_hits_neg_z.single(), other_rb, contact, false, !is_grounded.is_empty());
            }
        } else if let Ok((player_position, is_grounded)) = player_bodies.get_mut(contact.entity2) {
            if let Ok(other_rb) = bodies.get_mut(contact.entity1) {
                handle_collision(player_position, player_shape_hits_x.single(), player_shape_hits_neg_x.single(), player_shape_hits_z.single(), player_shape_hits_neg_z.single(), other_rb, contact, true, !is_grounded.is_empty());
            }
        }
    }
}

fn handle_collision(mut player_position: Mut<Position>, player_stepping_x: &ShapeHits, player_stepping_neg_x: &ShapeHits, player_stepping_z: &ShapeHits, player_stepping_neg_z: &ShapeHits, other_rb: &RigidBody, contact: &Contact, inverse: bool, is_grounded: bool) {
    if contact.penetration <= Scalar::EPSILON || other_rb.is_kinematic() {
        return;
    }

    let normal_to_use = if inverse { contact.normal * -1. } else { contact.normal };

    if normal_to_use.y.abs() < 0.1 && is_grounded {
        let corresponding_shape_hits: Option<&ShapeHits>;

        if normal_to_use == Vec3::X {
            corresponding_shape_hits = Some(player_stepping_x);
        } else if normal_to_use == Vec3::NEG_X {
            corresponding_shape_hits = Some(player_stepping_neg_x);
        } else if normal_to_use == Vec3::Z {
            corresponding_shape_hits = Some(player_stepping_z);
        } else if normal_to_use == Vec3::NEG_Z {
            corresponding_shape_hits = Some(player_stepping_neg_z);
        } else {
            corresponding_shape_hits = None;
        }

        if corresponding_shape_hits.is_some() {
            if !corresponding_shape_hits.unwrap().is_empty() {
                let hit = corresponding_shape_hits.unwrap().as_slice().first().unwrap();
                if hit.time_of_impact > 0. {
                    player_position.0.y += (STEP_HEIGHT + VOXEL_SIZE / 2.) - hit.time_of_impact;
                    return;
                }
            }
        }
    }

    player_position.0 -= normal_to_use * contact.penetration;
}