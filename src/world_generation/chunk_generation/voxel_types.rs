use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    usize,
};

use bevy::render::render_resource::{ShaderSize, ShaderType};

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
    [Vec4<u32>; ((CHUNK_SIZE[0] + 2) * (CHUNK_SIZE[1] + 2) * (CHUNK_SIZE[2] + 2) / 4 + 3) / 4];
pub type VoxelPalette = [Vec4<u32>; 8];

pub struct VoxelData {
    pub array: VoxelArray,
    pub palette: VoxelPalette,
    block_map: HashMap<BlockType, u8>,
}

impl Default for VoxelData {
    fn default() -> Self {
        Self {
            array: [Vec4::<u32>::default();
                ((CHUNK_SIZE[0] + 2) * (CHUNK_SIZE[1] + 2) * (CHUNK_SIZE[2] + 2) / 4 + 3) / 4],
            palette: [Vec4::<u32>::default(); 8],
            block_map: Default::default(),
        }
    }
}

impl VoxelData {
    pub fn is_air(&self, x: usize, y: usize, z: usize) -> bool {
        let (outer_index, inner_index) = Self::position_to_indexes(x, y, z);
        let divided_index = inner_index / 4;
        let index_remainder = inner_index % 4;

        let mask: u32 = 0b11111111 << (index_remainder * 8);

        (self.array[outer_index][divided_index] & mask) == 0
    }

    pub fn get_block(&self, x: usize, y: usize, z: usize) -> BlockType {
        let (outer_index, inner_index) = Self::position_to_indexes(x, y, z);
        let divided_index = inner_index / 4;
        let index_remainder = inner_index % 4;

        let mask: u32 = 0b11111111 << index_remainder;

        let map_index = ((self.array[outer_index][divided_index] & mask) >> index_remainder) as u8;

        *self
            .block_map
            .iter()
            .find_map(|(key, value)| if *value == map_index { Some(key) } else { None })
            .unwrap_or(&BlockType::Air)
    }

    pub fn set_block(&mut self, x: usize, y: usize, z: usize, block: BlockType) {
        let (outer_index, inner_index) = Self::position_to_indexes(x, y, z);
        let divided_index = inner_index / 4;
        let index_remainder = inner_index % 4;

        let previous = self.array[outer_index][divided_index];

        let palette_index = self.get_palette_index(block) as u32;

        self.array[outer_index][divided_index] =
            previous | (palette_index << (index_remainder * 8));
    }

    fn get_palette_index(&mut self, block: BlockType) -> u8 {
        if block == BlockType::Air {
            return 0;
        }

        if let Some(palette_index) = self.block_map.get(&block) {
            return *palette_index;
        }

        let current_index = self.block_map.len() as u8 + 1;

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

    fn position_to_indexes(x: usize, y: usize, z: usize) -> (usize, usize) {
        let index = x + (y * (CHUNK_SIZE[0] + 2)) + (z * (CHUNK_SIZE[0] + 2) * (CHUNK_SIZE[1] + 2));
        (index / 16, index % 16)
    }
}
