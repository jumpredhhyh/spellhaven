use noise::permutationtable::PermutationTable;
use noise::{NoiseFn, Seedable};
use crate::fractal_open_simplex::noise;

#[derive(Clone, Copy, Debug)]
pub struct Roughness {
    seed: u32,
    frequency: f64,
    amplitude: f64,
    permutation_table: PermutationTable,
}

impl Roughness {
    pub const DEFAULT_SEED: u32 = 0;

    pub fn new(seed: u32, frequency: f64, amplitude: f64) -> Self {
        Self {
            seed,
            frequency,
            amplitude,
            permutation_table: PermutationTable::new(seed),
        }
    }
}

impl Seedable for Roughness {
    fn set_seed(self, seed: u32) -> Self {
        if self.seed == seed {
            return self;
        }

        Self {
            seed,
            frequency: self.frequency,
            amplitude: self.amplitude,
            permutation_table: self.permutation_table,
        }
    }

    fn seed(&self) -> u32 {
        self.seed
    }
}

impl NoiseFn<i32, 2usize> for Roughness {
    fn get(&self, point: [i32; 2]) -> f64 {
        noise(point[0] as f64, point[1]as f64, self.frequency, self.amplitude, &self.permutation_table) - 0.15
    }
}