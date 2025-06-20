#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    pbr_functions::alpha_discard,
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

#import bevy_pbr::mesh_functions::{get_world_from_local, mesh_position_local_to_clip, mesh_normal_local_to_world}
#import bevy_pbr::pbr_functions::{calculate_view, prepare_world_normal}
#import bevy_pbr::mesh_view_bindings
#import bevy_pbr::mesh_bindings
#import bevy_pbr::mesh_bindings::mesh
#import bevy_pbr::pbr_types::pbr_input_new
#import bevy_pbr::prepass_utils

struct TerrainMaterial {
    palette: array<vec4<u32>, 128>,
    chunk_blocks: array<vec4<u32>, ((66 * 66 * 66 / 4 + 3) / 4)>,
    chunk_pos: vec3<i32>,
    chunk_lod: i32,
    min_chunk_height: i32,
};

@group(2) @binding(100)
var<uniform> terrain_material: TerrainMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) scale: vec2<f32>,
    @location(4) i_color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec4<f32>,
    @location(2) blend_color: vec3<f32>,
    @location(3) ambient: f32,
    @location(4) instance_index: u32,
};

@vertex
fn vertex(vertex: Vertex) -> VertexOutput {
    let position = vertex.position;
    var out: VertexOutput;

    out.clip_position = mesh_position_local_to_clip(
        get_world_from_local(0),
        vec4<f32>(position, 1.0),
    );
    out.blend_color = vertex.i_color;
    out.world_normal = vertex.normal;
    out.world_position = vec4<f32>(vertex.position, 1.0);
    out.ambient = 1.0;
    out.instance_index = vertex.instance_index;
    return out;
}

fn pcg(n: u32) -> u32 {
    var h = n * 747796405u + 2891336453u;
    h = ((h >> ((h >> 28u) + 4u)) ^ h) * 277803737u;
    return (h >> 22u) ^ h;
}

@fragment
fn fragment(input: VertexOutput) -> FragmentOutput {
    var pbr_input = pbr_input_new();

    pbr_input.flags = mesh[input.instance_index].flags;

    pbr_input.V = calculate_view(input.world_position, false);
    pbr_input.frag_coord = input.clip_position;
    pbr_input.world_position = input.world_position;

    pbr_input.world_normal = prepare_world_normal(
        input.world_normal,
        false,
        false,
    );
#ifdef LOAD_PREPASS_NORMALS
    pbr_input.N = prepass_utils::prepass_normal(input.clip_position, 0u);
#else
    pbr_input.N = normalize(pbr_input.world_normal);
#endif

    pbr_input.material.base_color = vec4<f32>(input.blend_color * input.ambient, 1.0);

    // pbr_input.material.reflectance = 0.0;
    // pbr_input.material.perceptual_roughness = 0.0;
    // pbr_input.material.metallic = 0.0;
    // pbr_input.material.metallic = 1.0;


#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);
#endif

    return out;

    // var voxel_size = 0.25;
    // var chunk_size = f32(64 * terrain_material.chunk_lod) * voxel_size;

    // var adjusted_world_pos = (input.world_position.xyz + voxel_size * f32(terrain_material.chunk_lod - 1) - (input.world_normal.xyz * (voxel_size * f32(terrain_material.chunk_lod)) * voxel_size)) - vec3<f32>(0, f32(terrain_material.min_chunk_height) * voxel_size * f32(terrain_material.chunk_lod), 0);

    // var chunk_pos_no_height = terrain_material.chunk_pos * vec3<i32>(1, 0, 1);

    // var voxel_pos = vec3<u32>((adjusted_world_pos / chunk_size - (vec3<f32>(chunk_pos_no_height) / f32(terrain_material.chunk_lod))) * 64);

    // var blocks_index = voxel_pos.x + voxel_pos.y * 66 + voxel_pos.z * 66 * 66;
    // var outer_index = blocks_index / 16;
    // var inner_index = blocks_index % 16;
    // var number = terrain_material.chunk_blocks[outer_index][inner_index / 4];
    // var number_mask: u32 = 0xffu << (inner_index % 4 * 8);
    // var palette_index = (number & number_mask) >> (inner_index % 4 * 8);
    // var palette_color = terrain_material.palette[palette_index / 4][palette_index % 4];

    // var red: f32 = f32((palette_color & 0xff0000u) >> 16) / 255;
    // var green: f32 = f32((palette_color & 0xff00u) >> 8) / 255;
    // var blue: f32 = f32((palette_color & 0xffu)) / 255;

    // var random1 = pcg(blocks_index);
    // var random2 = pcg(random1);
    // var random3 = pcg(random2);

    // red = red + (f32(random1) / 2147483647.5f - 1f) * 0.02;
    // green = green + (f32(random2) / 2147483647.5f - 1f) * 0.02;
    // blue = blue + (f32(random3) / 2147483647.5f - 1f) * 0.02;

    // out.color = out.color * vec4<f32>(red, green, blue, 1f);

    // return out;
}