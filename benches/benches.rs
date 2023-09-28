use brunch::{Bench, benches};
use spellhaven::mesh_generation::generate_mesh;
use spellhaven::voxel_generation::generate_voxels;

fn main() {
    let voxels = generate_voxels([0, 0]);

    benches!(
        inline:

        Bench::new("voxel_generation")
            .run(|| generate_voxels([0, 0])),

        Bench::new("mesh_generation")
            .run(|| generate_mesh(voxels)),
    );
}