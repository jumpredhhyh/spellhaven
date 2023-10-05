use std::sync::Arc;
use std::time::Instant;
use spellhaven::generation_options::{GenerationAssets, GenerationOptions};
use spellhaven::voxel_generation::{generate_voxels, vox_data_to_structure_data};

fn main() {
    let data = create_data();
    let now = Instant::now();
    let options = &GenerationOptions::get_options(data);
    let elapsed = now.elapsed();
    let now = Instant::now();
    println!("Elapsed: {:.2?}", elapsed);
    let chunk = generate_voxels([0, 0, 0], options);
    let elapsed = now.elapsed();
    println!("Elapsed: {:.2?}", elapsed);
    println!("{}", chunk.1);
}

fn create_data() -> Arc<GenerationAssets> {
    Arc::new(GenerationAssets {
        tree: vox_data_to_structure_data(&vox_format::from_file("assets/tree.vox").unwrap()),
        tree_house: vox_data_to_structure_data(&vox_format::from_file("assets/tree_house.vox").unwrap()),
    })
}