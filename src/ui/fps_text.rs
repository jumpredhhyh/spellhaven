use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::{Component, Query, Res, Text, With};

use crate::world_generation::chunk_generation::ChunkTriangles;

#[derive(Component)]
pub struct FpsText;

pub fn update_fps_ui(
    mut texts: Query<&mut Text, With<FpsText>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    for mut text in &mut texts {
        if let Some(fps) = diagnostics
            .get(&FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            text.0 = format!("FPS: {:.0}", fps);
        }
    }
}

#[derive(Component)]
pub struct TriangleText;

pub fn update_triangle_ui(
    mut texts: Query<&mut Text, With<TriangleText>>,
    triangle_count: Res<ChunkTriangles>,
) {
    for mut text in &mut texts {
        text.0 = format!(
            "Triangles: {}, Total: {}",
            triangle_count
                .0
                .map(|x| x
                    .to_string()
                    .as_bytes()
                    .rchunks(3)
                    .rev()
                    .map(std::str::from_utf8)
                    .collect::<Result<Vec<&str>, _>>()
                    .unwrap()
                    .join("'"))
                .join(", "),
            triangle_count.0.iter().sum::<u64>()
        );
    }
}
