use noise::core::open_simplex::open_simplex_2d;
use noise::permutationtable::PermutationTable;
use noise::{NoiseFn, Seedable};

#[derive(Clone, Copy, Debug)]
pub struct FractalOpenSimplex<R> where R: NoiseFn<i32, 2usize> {
    seed: u32,
    frequency: f64,
    amplitude: f64,
    octaves: i32,
    lacunarity: f64,
    persistence: f64,
    permutation_table: PermutationTable,
    roughness: R,
}

impl<R> FractalOpenSimplex<R> where R: NoiseFn<i32, 2usize> {
    pub const DEFAULT_SEED: u32 = 0;

    pub fn new(seed: u32, frequency: f64, amplitude: f64, octaves: i32, lacunarity: f64, persistence: f64, roughness: R) -> Self {
        Self {
            seed,
            frequency,
            amplitude,
            octaves,
            lacunarity,
            persistence,
            permutation_table: PermutationTable::new(seed),
            roughness,
        }
    }
}

impl<R> Seedable for FractalOpenSimplex<R> where R: NoiseFn<i32, 2usize> {
    fn set_seed(self, seed: u32) -> Self {
        if self.seed == seed {
            return self;
        }

        Self {
            seed,
            frequency: self.frequency,
            amplitude: self.amplitude,
            octaves: self.octaves,
            lacunarity: self.lacunarity,
            persistence: self.persistence,
            permutation_table: self.permutation_table,
            roughness: self.roughness
        }
    }

    fn seed(&self) -> u32 {
        self.seed
    }
}

impl<R> NoiseFn<i32, 2usize> for FractalOpenSimplex<R> where R: NoiseFn<i32, 2usize> {
    fn get(&self, point: [i32; 2]) -> f64 {
        let roughness = self.roughness.get(point);
        fractal_noise(point[0] as f64, point[1] as f64, self.frequency, self.amplitude, self.octaves, self.lacunarity, self.persistence + roughness, &self.permutation_table).max(2.)
    }
}

pub fn fractal_noise(x: f64, z: f64, frequency: f64, amplitude: f64, octaves: i32, lacunarity: f64, persistence: f64, hasher: &PermutationTable) -> f64 {
    let mut noise_value: f64 = 0.;

    for octave in 0..octaves {
        noise_value += noise(x, z, frequency * lacunarity.powi(octave), amplitude * persistence.powi(octave), hasher)
    }

    noise_value
}

pub fn noise(x: f64, z: f64, frequency: f64, amplitude: f64, hasher: &PermutationTable) -> f64 {
    (open_simplex_2d([x * frequency, z * frequency], hasher) + 0.5) * amplitude
}