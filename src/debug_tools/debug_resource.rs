use bevy::app::App;
use bevy::prelude::{Plugin, Reflect, Resource, ReflectResource};
use bevy_inspector_egui::quick::ResourceInspectorPlugin;

pub struct SpellhavenDebugPlugin;

impl Plugin for SpellhavenDebugPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<SpellhavenDebug>()
            .register_type::<SpellhavenDebug>()
            .add_plugins(ResourceInspectorPlugin::<SpellhavenDebug>::default());
    }
}

#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct SpellhavenDebug {
    pub unlock_camera: bool,
    pub show_path_debug: bool,
    pub path_circle_radius: f32,
    pub path_show_range: i32,
}

impl Default for SpellhavenDebug {
    fn default() -> Self {
        Self {
            unlock_camera: false,
            show_path_debug: false,
            path_circle_radius: 1.,
            path_show_range: 500,
        }
    }
}