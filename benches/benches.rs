use brunch::{Bench, benches};
use spellhaven::mesh_generation::generate_mesh;
use spellhaven::voxel_generation::{generate_voxels, vox_data_to_blocks};

fn main() {
    let model = vox_data_to_blocks(vox_format::from_file("assets/tree.vox").unwrap());
    let voxels = generate_voxels([0, 0], &model);

    benches!(
        inline:

        Bench::new("voxel_generation")
            .run(|| generate_voxels([0, 0], &model)),

        Bench::new("mesh_generation")
            .run(|| generate_mesh(voxels)),
    );
}