use brunch::{Bench, benches};
use spellhaven::chunk_generation::GenerationOptions;
use spellhaven::mesh_generation::generate_mesh;
use spellhaven::voxel_generation::{generate_voxels, vox_data_to_blocks};

fn main() {
    let voxels = generate_voxels([0, 0, 0], &GenerationOptions::get_options());

    benches!(
        inline:

        Bench::new("voxel_generation")
            .run(|| generate_voxels([0, 0, 0], &GenerationOptions::get_options())),

        Bench::new("mesh_generation")
            .run(|| generate_mesh(voxels)),
    );
}