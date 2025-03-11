use std::{cell::RefCell, collections::HashMap, rc::Rc};

use bevy::{
    app::{Plugin, Startup},
    asset::{AssetServer, Assets, Handle},
    ecs::{
        component::Component,
        entity::Entity,
        system::{Commands, Res, ResMut},
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

    commands.spawn(WfcTilemap { _tiles: hash });
}

#[derive(Component)]
struct WfcTile {
    _position: IVec2,
    _state: Flags,
}

#[derive(Component)]
struct WfcTilemap {
    _tiles: HashMap<IVec2, Entity>,
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
            _position: position,
            _state: Flags::all(),
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
    fn _remove_state(
        &mut self,
        state_to_remove: Flags,
        bitmap: Rc<RefCell<HashMap<IVec2, RefCell<WfcTile>>>>,
    ) {
        let removed_state = self._state & state_to_remove;
        self._state &= !state_to_remove;

        for set_flag in removed_state {
            let cloned_bimap = bitmap.clone();

            match set_flag {
                Flags::Barrel => {
                    bitmap
                        .clone()
                        .borrow_mut()
                        .entry(self._position - IVec2::Y)
                        .and_modify(move |f| {
                            f.borrow_mut()._remove_state(
                                !(Flags::FloorTL | Flags::FloorTM | Flags::FloorTR),
                                cloned_bimap,
                            )
                        });
                }
                Flags::BarrelBroken => {
                    bitmap
                        .borrow_mut()
                        .entry(self._position - IVec2::Y)
                        .and_modify(|f| {
                            f.borrow_mut()._state &=
                                Flags::FloorTL | Flags::FloorTM | Flags::FloorTR
                        });
                }
                Flags::PoleTop => {
                    bitmap
                        .borrow_mut()
                        .entry(self._position - IVec2::Y)
                        .and_modify(|f| {
                            f.borrow_mut()._state &= Flags::PoleMiddle | Flags::PoleBottom
                        });
                }
                Flags::PoleMiddle => {
                    bitmap
                        .borrow_mut()
                        .entry(self._position - IVec2::Y)
                        .and_modify(|f| {
                            f.borrow_mut()._state &= Flags::PoleMiddle | Flags::PoleBottom
                        });
                    bitmap
                        .borrow_mut()
                        .entry(self._position + IVec2::Y)
                        .and_modify(|f| f.borrow_mut()._state &= Flags::PoleTop);
                }
                Flags::PoleBottom => {
                    bitmap
                        .borrow_mut()
                        .entry(self._position - IVec2::Y)
                        .and_modify(|f| {
                            f.borrow_mut()._state &=
                                Flags::FloorTL | Flags::FloorTM | Flags::FloorTR
                        });
                    bitmap
                        .borrow_mut()
                        .entry(self._position + IVec2::Y)
                        .and_modify(|f| {
                            f.borrow_mut()._state &= Flags::PoleTop | Flags::PoleMiddle
                        });
                }
                Flags::FloorTL => {
                    bitmap
                        .borrow_mut()
                        .entry(self._position - IVec2::Y)
                        .and_modify(|f| {
                            f.borrow_mut()._state &=
                                Flags::FloorTL | Flags::FloorTM | Flags::FloorTR
                        });
                    bitmap
                        .borrow_mut()
                        .entry(self._position + IVec2::Y)
                        .and_modify(|f| {
                            f.borrow_mut()._state &= Flags::PoleTop | Flags::PoleMiddle
                        });
                }
                _ => {}
            }
        }
    }
}
