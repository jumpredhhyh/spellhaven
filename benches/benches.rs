use std::sync::Arc;
use brunch::{Bench, benches};
use spellhaven::generation_options::{GenerationAssets, GenerationOptions};
use spellhaven::mesh_generation::generate_mesh;
use spellhaven::voxel_generation::{generate_voxels, vox_data_to_structure_data};

fn main() {
    let arc = Arc::new(GenerationAssets {
        tree: vox_data_to_structure_data(&vox_format::from_file("assets/tree.vox").unwrap()),
        tree_house: vox_data_to_structure_data(&vox_format::from_file("assets/tree_house.vox").unwrap()),
    });
    let voxels = generate_voxels([0, 0, 0], &GenerationOptions::get_options(Arc::clone(&arc)));

    benches!(
        inline:

        Bench::new("voxel_generation")
            .run(|| generate_voxels([0, 0, 0], &GenerationOptions::get_options(Arc::clone(&arc)))),

        Bench::new("mesh_generation")
            .run(|| generate_mesh(voxels)),
    );
}