use std::num::Wrapping;
use noise::{NoiseFn, Seedable};

#[derive(Clone, Copy, Debug)]
pub struct WhiteNoise {
    seed: u32
}

impl WhiteNoise {
    pub const DEFAULT_SEED: u32 = 1;

    pub fn new(seed: u32) -> Self {
        Self{
            seed
        }
    }
}

impl Default for WhiteNoise {
    fn default() -> Self {
        Self::new(Self::DEFAULT_SEED)
    }
}

impl Seedable for WhiteNoise {
    fn set_seed(self, seed: u32) -> Self {
        if self.seed == seed {
            return self;
        }

        Self {
            seed
        }
    }

    fn seed(&self) -> u32 {
        self.seed
    }
}

impl NoiseFn<f64, 2usize> for WhiteNoise {
    fn get(&self, point: [f64; 2]) -> f64 {
        ((hash(Wrapping((point[0] as i64 + i32::MAX as i64) as u32)) * Wrapping(self.seed) * hash(Wrapping((point[1] as i64 + i32::MAX as i64) as u32))).0 as f64 - i32::MAX as f64) / i32::MAX as f64
    }
}

// https://stackoverflow.com/questions/664014/what-integer-hash-function-are-good-that-accepts-an-integer-hash-key
fn hash(mut x: Wrapping<u32>) -> Wrapping<u32>  {
    x = ((x >> 16) ^ x) * Wrapping(0x45d9f3b);
    x = ((x >> 16) ^ x) * Wrapping(0x45d9f3b);
    (x >> 16) ^ x
}
