use bevy::app::App;
use bevy::hierarchy::DespawnRecursiveExt;
use bevy::prelude::{
    Commands, Component, Entity, Plugin, Query, Res, Time, Transform, Update, Vec3, With, Without,
};

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (animate_spawn_animation, animate_despawn_animation));
    }
}

#[derive(Component, Default)]
pub struct SpawnAnimation(f32, Option<Vec3>);

#[derive(Component, Default)]
pub struct DespawnAnimation(f32, Option<Vec3>);

fn animate_spawn_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut spawn_animations: Query<(Entity, &mut Transform, &mut SpawnAnimation)>,
) {
    for (entity, mut transform, mut spawn_animation) in &mut spawn_animations {
        if spawn_animation.0 == 0. && spawn_animation.1.is_none() {
            spawn_animation.1 = Some(transform.translation);
        }

        transform.translation.y =
            spawn_animation.1.unwrap().y + 40. * (1. - (1. - spawn_animation.0.min(1.)).powi(2));

        if spawn_animation.0 >= 1. {
            commands.entity(entity).remove::<SpawnAnimation>();
            continue;
        }

        spawn_animation.0 += time.delta_secs();
    }
}

fn animate_despawn_animation(
    mut commands: Commands,
    time: Res<Time>,
    mut despawn_animations: Query<(Entity, &mut Transform, &mut DespawnAnimation)>,
    mut despawn_animations_no_transform: Query<
        Entity,
        (With<DespawnAnimation>, Without<Transform>),
    >,
) {
    for entity in &mut despawn_animations_no_transform {
        commands.entity(entity).despawn_recursive();
    }

    for (entity, mut transform, mut despawn_animation) in &mut despawn_animations {
        if despawn_animation.0 == 0. && despawn_animation.1.is_none() {
            despawn_animation.1 = Some(transform.translation);
        }

        transform.translation.y =
            despawn_animation.1.unwrap().y - 40. * despawn_animation.0.min(1.).powi(2);

        if despawn_animation.0 >= 1. {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        despawn_animation.0 += time.delta_secs();
    }
}
