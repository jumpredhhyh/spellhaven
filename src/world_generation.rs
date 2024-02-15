pub mod chunk_generation;
pub mod chunk_loading;
pub mod voxel_world;
pub mod generation_options;

use bevy::app::App;
use bevy::prelude::Plugin;
use crate::world_generation::chunk_generation::ChunkGenerationPlugin;

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChunkGenerationPlugin);
    }
}
