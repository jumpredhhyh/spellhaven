#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}

struct TerrainMaterial {
    palette: array<vec4<u32>, 8>,
    chunk_blocks: array<vec4<u32>, ((66 * 66 * 66 / 4 + 3) / 4)>,
    chunk_pos: vec3<i32>,
    chunk_lod: i32,
    min_chunk_height: i32,
};

@group(2) @binding(100)
var<uniform> terrain_material: TerrainMaterial;

fn pcg(n: u32) -> u32 {
    var h = n * 747796405u + 2891336453u;
    h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
    return (h >> 22u) ^ h;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
     // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_from_standard_material(in, is_front);

    // we can optionally modify the input before lighting and alpha_discard is applied
    pbr_input.material.base_color.b = pbr_input.material.base_color.r;

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // we can optionally modify the lit color before post-processing is applied
    // out.color = vec4<f32>(vec4<u32>(out.color * f32(my_extended_material.quantize_steps))) / f32(my_extended_material.quantize_steps);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    // we can optionally modify the final result here

    var voxel_size = 0.5;
    var chunk_size = f32(64 * terrain_material.chunk_lod) * voxel_size;

    var adjusted_world_pos = (in.world_position.xyz + 0.5 * f32(terrain_material.chunk_lod - 1) - (in.world_normal.xyz * (voxel_size * f32(terrain_material.chunk_lod)) * 0.5)) - vec3<f32>(0, f32(terrain_material.min_chunk_height) * voxel_size * f32(terrain_material.chunk_lod), 0);

    var chunk_pos_no_height = terrain_material.chunk_pos * vec3<i32>(1, 0, 1);

    var voxel_pos = vec3<u32>((adjusted_world_pos / chunk_size - (vec3<f32>(chunk_pos_no_height) / f32(terrain_material.chunk_lod))) * 64);

    var blocks_index = voxel_pos.x + voxel_pos.y * 66 + voxel_pos.z * 66 * 66;
    var outer_index = blocks_index / 16;
    var inner_index = blocks_index % 16;
    var number = terrain_material.chunk_blocks[outer_index][inner_index / 4];
    var number_mask: u32 = 0xffu << (inner_index % 4 * 8);
    var palette_index = (number & number_mask) >> (inner_index % 4 * 8);
    var palette_color = terrain_material.palette[palette_index / 4][palette_index % 4];

    var red: f32 = f32((palette_color & 0xff0000u) >> 16) / 255;
    var green: f32 = f32((palette_color & 0xff00u) >> 8) / 255;
    var blue: f32 = f32((palette_color & 0xffu)) / 255;

    var random1 = pcg(blocks_index);
    var random2 = pcg(random1);
    var random3 = pcg(random2);

    red = red + (f32(random1) / 2147483647.5f - 1f) * 0.02;
    green = green + (f32(random2) / 2147483647.5f - 1f) * 0.02;
    blue = blue + (f32(random3) / 2147483647.5f - 1f) * 0.02;

    out.color = out.color * vec4<f32>(red, green, blue, 1f);

    //out.color = vec4<f32>(voxel_pos.xyzz) / 64;

    return out;
}