use std::time::Instant;
use spellhaven::generation_options::{GenerationOptions, GenerationOptionsResource};
use spellhaven::mesh_generation::generate_mesh;
use spellhaven::voxel_generation::generate_voxels;

fn main() {
    let data = GenerationOptionsResource::default().0;
    let mut instant = Instant::now();

    instant = time_stamp(instant);

    let chunk = generate_voxels([0, 0, 0], &data);

    instant = time_stamp(instant);

    let _mesh = generate_mesh(chunk);

    time_stamp(instant);
}

fn time_stamp(instant: Instant) -> Instant {
    let elapsed = instant.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    Instant::now()
}