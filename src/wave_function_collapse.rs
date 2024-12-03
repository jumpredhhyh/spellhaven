use std::collections::HashMap;

use bevy::{
    app::{Plugin, Startup},
    asset::{AssetServer, Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Res, ResMut},
        world::World,
    },
    math::{IVec2, Vec2, Vec3},
    prelude::default,
    render::texture::Image,
    sprite::{SpriteBundle, TextureAtlas, TextureAtlasLayout},
    transform::components::Transform,
};

pub struct WaveFunctionCollapsePlugin;

impl Plugin for WaveFunctionCollapsePlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugins(FlatCameraPlugin)
            .add_systems(Startup, startup);
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    let texture_handle: Handle<Image> = asset_server.load("wfc_2d/cave_tileset.png");
    let texture_atlas = TextureAtlasLayout::from_grid(Vec2::new(32.0, 32.0), 10, 7, None, None);
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let indicies = [
        [56, 66, 47, 3],
        [40, 41, 42, 45],
        [50, 51, 52, 55],
        [60, 61, 62, 65],
    ];

    let mut hash = HashMap::new();

    for (y, i) in indicies.iter().enumerate() {
        for (x, j) in i.iter().enumerate() {
            spawn_sprite(
                &mut commands,
                &texture_handle,
                &texture_atlas_handle,
                *j,
                IVec2::new(x as i32 * 64, -(y as i32 * 64)),
                &mut hash,
            );
        }
    }

    commands.spawn(WfcTilemap { tiles: hash });
}

#[derive(Component)]
struct WfcTile {
    position: IVec2,
    state: Flags,
}

#[derive(Component)]
struct WfcTilemap {
    tiles: HashMap<IVec2, Entity>,
}

fn spawn_sprite(
    commands: &mut Commands,
    texture: &Handle<Image>,
    texture_atlas_handle: &Handle<TextureAtlasLayout>,
    index: usize,
    position: IVec2,
    tile_map: &mut HashMap<IVec2, Entity>,
) {
    let entity = commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec3::new(position.x as f32, position.y as f32, 0.),
                scale: Vec3::splat(2.0),
                ..default()
            },
            texture: texture.clone(),
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_handle.clone(),
            index: index,
        },
        WfcTile {
            position,
            state: Flags::all(),
        },
    ));

    tile_map.insert(position, entity.id());
}

use bitflags::bitflags;

use crate::flat_camera::FlatCameraPlugin;

bitflags! {
    #[derive(Clone, Copy, PartialEq)]
    pub struct Flags: u32 {
        const None = 0;
        const Barrel = 1 << 0;
        const BarrelBroken = 1 << 1;
        const Torch = 1 << 2;
        const FloorTL = 1 << 3;
        const FloorTM = 1 << 4;
        const FloorTR = 1 << 5;
        const FloorML = 1 << 6;
        const FloorMM = 1 << 7;
        const FloorMR = 1 << 8;
        const FloorBL = 1 << 9;
        const FloorBM = 1 << 10;
        const FloorBR = 1 << 11;
        const PoleTop = 1 << 12;
        const PoleMiddle = 1 << 13;
        const PoleBottom = 1 << 14;
        const Air = 1 << 15;
    }
}

impl WfcTile {
    fn remove_state(&mut self, state_to_remove: Flags, bitmap: &mut HashMap<IVec2, &mut WfcTile>) {
        let removed_state = self.state & state_to_remove;
        self.state &= !state_to_remove;

        for set_flag in removed_state {
            // match set_flag {
            //     Flags::Barrel => {
            //         bitmap.entry(self.position - IVec2::Y).and_modify(move |f| {
            //             f.remove_state(!(Flags::FloorTL | Flags::FloorTM | Flags::FloorTR), bitmap)
            //         });
            //     }
            //     Flags::BarrelBroken => {
            //         bitmap.entry(self.position - IVec2::Y).and_modify(|f| {
            //             f.state &= Flags::FloorTL | Flags::FloorTM | Flags::FloorTR
            //         });
            //     }
            //     Flags::PoleTop => {
            //         bitmap
            //             .entry(self.position - IVec2::Y)
            //             .and_modify(|f| f.state &= Flags::PoleMiddle | Flags::PoleBottom);
            //     }
            //     Flags::PoleMiddle => {
            //         bitmap
            //             .entry(self.position - IVec2::Y)
            //             .and_modify(|f| f.state &= Flags::PoleMiddle | Flags::PoleBottom);
            //         bitmap
            //             .entry(self.position + IVec2::Y)
            //             .and_modify(|f| f.state &= Flags::PoleTop);
            //     }
            //     Flags::PoleBottom => {
            //         bitmap.entry(self.position - IVec2::Y).and_modify(|f| {
            //             f.state &= Flags::FloorTL | Flags::FloorTM | Flags::FloorTR
            //         });
            //         bitmap
            //             .entry(self.position + IVec2::Y)
            //             .and_modify(|f| f.state &= Flags::PoleTop | Flags::PoleMiddle);
            //     }
            //     Flags::FloorTL => {
            //         bitmap.entry(self.position - IVec2::Y).and_modify(|f| {
            //             f.state &= Flags::FloorTL | Flags::FloorTM | Flags::FloorTR
            //         });
            //         bitmap
            //             .entry(self.position + IVec2::Y)
            //             .and_modify(|f| f.state &= Flags::PoleTop | Flags::PoleMiddle);
            //     }
            //     _ => {}
            // }
        }
    }
}
