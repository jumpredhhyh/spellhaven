use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    usize,
};

use bevy::{
    log::error,
    math::IVec3,
    render::render_resource::{ShaderSize, ShaderType},
};

use super::{BlockType, CHUNK_SIZE};

#[derive(Debug, Clone, ShaderType, Default, Copy)]
pub struct Vec4<T: ShaderSize> {
    one: T,
    two: T,
    three: T,
    four: T,
}

impl<T: ShaderSize> Index<usize> for Vec4<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.one,
            1 => &self.two,
            2 => &self.three,
            3 => &self.four,
            _ => panic!("Outisde of Range!"),
        }
    }
}

impl<T: ShaderSize> IndexMut<usize> for Vec4<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        match index {
            0 => &mut self.one,
            1 => &mut self.two,
            2 => &mut self.three,
            3 => &mut self.four,
            _ => panic!("Outisde of Range!"),
        }
    }
}

pub type VoxelArray =
    [Vec4<u32>; ((CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) / 4 + 3) / 4];
pub type VoxelPalette = [Vec4<u32>; 128];

pub struct VoxelData {
    pub array: VoxelArray,
    pub palette: VoxelPalette,
    block_map: HashMap<BlockType, u16>,
}

impl Default for VoxelData {
    fn default() -> Self {
        Self {
            array: [Vec4::<u32>::default();
                ((CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2) / 4 + 3) / 4],
            palette: [Vec4::<u32>::default(); 128],
            block_map: Default::default(),
        }
    }
}

impl VoxelData {
    pub fn is_air<T: Into<IVec3>>(&self, position: T) -> bool {
        let (outer_index, inner_index) = Self::position_to_indexes(position);
        let divided_index = inner_index / 4;
        let index_remainder = inner_index % 4;

        let mask: u32 = 0b11111111 << (index_remainder * 8);

        (self.array[outer_index][divided_index] & mask) == 0
    }

    pub fn get_block<T: Into<IVec3>>(&self, position: T) -> BlockType {
        let (outer_index, inner_index) = Self::position_to_indexes(position);
        let divided_index = inner_index / 4;
        let index_remainder = inner_index % 4;

        let mask: u32 = 0b11111111 << index_remainder;

        let map_index = ((self.array[outer_index][divided_index] & mask) >> index_remainder) as u16;

        *self
            .block_map
            .iter()
            .find_map(|(key, value)| if *value == map_index { Some(key) } else { None })
            .unwrap_or(&BlockType::Air)
    }

    pub fn set_block<T: Into<IVec3>>(&mut self, position: T, block: BlockType) {
        let (outer_index, inner_index) = Self::position_to_indexes(position);
        let divided_index = inner_index / 4;
        let index_remainder = inner_index % 4;

        let previous_mask = !(0xffu32 << (index_remainder * 8));

        let previous = self.array[outer_index][divided_index] & previous_mask;

        let palette_index = self.get_palette_index(block) as u32;

        if palette_index > 0xff {
            error!("Palette index too high!");
        }

        self.array[outer_index][divided_index] =
            previous | (palette_index.min(0xffu32) << (index_remainder * 8));
    }

    fn get_palette_index(&mut self, block: BlockType) -> u16 {
        if block == BlockType::Air {
            return 0;
        }

        if let Some(palette_index) = self.block_map.get(&block) {
            return *palette_index;
        }

        let current_index = self.block_map.len() as u16 + 1;
        // if current_index >= 32 {
        //     return 31;
        // }

        let color = block.get_color();

        let compressed_color = (((color[0] * 255.) as u32) << 16)
            | (((color[1] * 255.) as u32) << 8)
            | ((color[2] * 255.) as u32);

        let palette_index = current_index / 4;
        let palette_remainder = current_index % 4;

        self.palette[palette_index as usize][palette_remainder as usize] = compressed_color;
        self.block_map.insert(block, current_index);

        current_index
    }

    fn position_to_indexes<T: Into<IVec3>>(position: T) -> (usize, usize) {
        let position: IVec3 = position.into();
        let index = position.x as usize
            + (position.y as usize * (CHUNK_SIZE + 2))
            + (position.z as usize * (CHUNK_SIZE + 2) * (CHUNK_SIZE + 2));
        (index / 16, index % 16)
    }
}
