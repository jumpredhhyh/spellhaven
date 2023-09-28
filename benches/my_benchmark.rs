use criterion::{criterion_group, criterion_main, Criterion, black_box};
use spellhaven::mesh_generation::generate_mesh;
use spellhaven::voxel_generation::generate_voxels;

fn criterion_benchmark(c: &mut Criterion) {
    let voxels = generate_voxels([0, 0]);
    c.bench_function("mesh generation", |b| b.iter(|| generate_mesh(voxels)));
    c.bench_function("voxel generation", |b| b.iter(|| generate_voxels(black_box([0, 0]))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);