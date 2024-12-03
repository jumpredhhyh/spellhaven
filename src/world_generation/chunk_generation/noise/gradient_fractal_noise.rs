use std::f64::consts::E;

use noise::{math::vectors::Vector2, MultiFractal, NoiseFn, Seedable};

pub struct GFT<T> {
    pub octaves: usize,
    pub frequency: f64,
    pub lacunarity: f64,
    pub persistence: f64,
    pub gradient: f64,
    pub amplitude: f64,

    sources: Vec<T>,
    seed: u32,
    scale_factor: f64,
}

impl<T> GFT<T>
where
    T: Default + Seedable,
{
    pub const DEFAULT_OCTAVE_COUNT: usize = 6;
    pub const DEFAULT_FREQUENCY: f64 = 1.0;
    pub const DEFAULT_LACUNARITY: f64 = 2.0; //core::f64::consts::PI * 2.0 / 3.0;
    pub const DEFAULT_PERSISTENCE: f64 = 0.5;
    pub const DEFAULT_GRADIENT: f64 = 1.;
    pub const DEFAULT_AMPLITUDE: f64 = 1.;
    pub const DEFAULT_SEED: u32 = 0;

    pub fn new(seed: u32) -> Self {
        Self {
            seed,
            octaves: Self::DEFAULT_OCTAVE_COUNT,
            frequency: Self::DEFAULT_FREQUENCY,
            lacunarity: Self::DEFAULT_LACUNARITY,
            persistence: Self::DEFAULT_PERSISTENCE,
            gradient: Self::DEFAULT_GRADIENT,
            amplitude: Self::DEFAULT_AMPLITUDE,
            sources: build_sources(seed, Self::DEFAULT_OCTAVE_COUNT),
            scale_factor: Self::calc_scale_factor(
                Self::DEFAULT_PERSISTENCE,
                Self::DEFAULT_OCTAVE_COUNT,
            ),
        }
    }

    fn calc_scale_factor(persistence: f64, octaves: usize) -> f64 {
        let denom = (1..=octaves).fold(0.0, |acc, x| acc + persistence.powi(x as i32));

        1.0 / denom
    }
}

impl<T> Default for GFT<T>
where
    T: Default + Seedable,
{
    fn default() -> Self {
        Self::new(Self::DEFAULT_SEED)
    }
}

impl<T> MultiFractal for GFT<T>
where
    T: Default + Seedable,
{
    fn set_octaves(self, octaves: usize) -> Self {
        if self.octaves == octaves {
            return self;
        }

        Self {
            octaves,
            sources: build_sources(self.seed, octaves),
            scale_factor: Self::calc_scale_factor(self.persistence, octaves),
            ..self
        }
    }

    fn set_frequency(self, frequency: f64) -> Self {
        Self { frequency, ..self }
    }

    fn set_lacunarity(self, lacunarity: f64) -> Self {
        Self { lacunarity, ..self }
    }

    fn set_persistence(self, persistence: f64) -> Self {
        Self {
            persistence,
            scale_factor: Self::calc_scale_factor(persistence, self.octaves),
            ..self
        }
    }
}

impl<T> GFT<T> {
    pub fn set_amplitude(self, amplitude: f64) -> Self {
        Self { amplitude, ..self }
    }

    pub fn set_gradient(self, gradient: f64) -> Self {
        Self { gradient, ..self }
    }

    fn get_gradient_influence(&self, steepness: f64) -> f64 {
        E.powf(-(steepness * self.gradient).powi(3))
    }
}

/// 2-dimensional Fbm noise
impl<T> NoiseFn<f64, 2> for GFT<T>
where
    T: NoiseFn<f64, 2>,
{
    fn get(&self, point: [f64; 2]) -> f64 {
        let point = Vector2::from(point);
        let derivative_offset = 0.001;
        let offset_x_point = Vector2::new(derivative_offset, 0.);
        let offset_y_point = Vector2::new(0., derivative_offset);

        let mut result = 0.0;

        let mut total_derivative = Vector2::new(0., 0.);

        for x in 0..self.octaves as i32 {
            let frequency = self.frequency * self.lacunarity.powi(x);
            let amplitude = self.amplitude * self.persistence.powi(x + 1);

            // Get the signal.
            let noise_value = self.sources[x as usize].get((point * frequency).into_array());
            let noise_value_offset_x =
                self.sources[x as usize].get((point * frequency + offset_x_point).into_array());
            let noise_value_offset_y =
                self.sources[x as usize].get((point * frequency + offset_y_point).into_array());

            total_derivative += Vector2::new(
                (noise_value_offset_x - noise_value) / derivative_offset,
                (noise_value_offset_y - noise_value) / derivative_offset,
            );
            let steepness = total_derivative.magnitude();

            let gradience = self.get_gradient_influence(steepness);

            // Add the signal to the result.
            result += noise_value * gradience * amplitude;
        }

        // Scale the result into the [-1,1] range
        result * self.scale_factor
    }
}

fn build_sources<Source>(seed: u32, octaves: usize) -> Vec<Source>
where
    Source: Default + Seedable,
{
    let mut sources = Vec::with_capacity(octaves);
    for x in 0..octaves {
        let source = Source::default();
        sources.push(source.set_seed(seed + x as u32));
    }
    sources
}
