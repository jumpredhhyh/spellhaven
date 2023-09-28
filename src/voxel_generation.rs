use noise::core::open_simplex::open_simplex_2d;
use noise::permutationtable::PermutationTable;
use crate::chunk_generation::{BlockType, CHUNK_SIZE, LEVEL_OF_DETAIL};

pub fn generate_voxels(position: [i32; 2]) -> [[[BlockType; CHUNK_SIZE[2] + 2]; CHUNK_SIZE[1] + 2]; CHUNK_SIZE[0] + 2] {
    let mut blocks = [[[BlockType::Air; CHUNK_SIZE[2] + 2]; CHUNK_SIZE[1] + 2]; CHUNK_SIZE[0] + 2];
    let hasher = PermutationTable::new(0);
    let roughness_hasher = PermutationTable::new(1);

    for x in 0..CHUNK_SIZE[0] + 2 {
        for z in 0..CHUNK_SIZE[2] + 2 {
            let total_x = position[0] * CHUNK_SIZE[0] as i32 + x as i32;
            let total_z = position[1] * CHUNK_SIZE[2] as i32 + z as i32;

            let roughness = noise(total_x, total_z, 0.0015625 * LEVEL_OF_DETAIL as f64, 0.2 / LEVEL_OF_DETAIL as f64, &roughness_hasher) - 0.15;

            let noise_height = fractal_noise(total_x, total_z, 0.0035 * LEVEL_OF_DETAIL as f64, 128. / LEVEL_OF_DETAIL as f64, 7, 2., 0.5 + roughness, &hasher);

            for y in 0..noise_height.min((CHUNK_SIZE[1] + 1) as f64) as usize {
                blocks[x][y][z] = if y + 1 == noise_height.floor() as usize { BlockType::Grass } else { BlockType::Stone }
            }
        }
    }

    blocks
}

fn fractal_noise(x: i32, z: i32, frequency: f64, amplitude: f64, octaves: i32, lacunarity: f64, persistence: f64, hasher: &PermutationTable) -> f64 {
    let mut noise_value: f64 = 0.;

    for octave in 0..octaves {
        noise_value += noise(x, z, frequency * lacunarity.powi(octave), amplitude * persistence.powi(octave), hasher)
    }

    noise_value
}

fn noise(x: i32, z: i32, frequency: f64, amplitude: f64, hasher: &PermutationTable) -> f64 {
    (open_simplex_2d([x as f64 * frequency, z as f64 * frequency], hasher) + 0.5) * amplitude
}