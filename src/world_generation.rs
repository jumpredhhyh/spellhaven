pub mod chunk_generation;
pub mod chunk_loading;
pub mod generation_options;
pub mod voxel_world;

use crate::world_generation::chunk_generation::ChunkGenerationPlugin;
use bevy::app::App;
use bevy::prelude::Plugin;

pub struct WorldGenerationPlugin;

impl Plugin for WorldGenerationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ChunkGenerationPlugin);
    }
}
