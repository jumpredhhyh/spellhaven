use std::time::Duration;
use brunch::{Bench, benches};
use spellhaven::generation_options::{GenerationOptions, GenerationOptionsResource};
use spellhaven::mesh_generation::generate_mesh;
use spellhaven::voxel_generation::generate_voxels;

fn main() {
    let arc = GenerationOptionsResource::default().0;
    let voxels = generate_voxels([0, 0, 0], &arc);

    benches!(
        inline:

        Bench::new("voxel_generation").with_timeout(Duration::from_secs(20))
            .run(|| generate_voxels([0, 0, 0], &arc)),

        Bench::new("mesh_generation").with_timeout(Duration::from_secs(20))
            .run(|| generate_mesh(voxels)),
    );
}