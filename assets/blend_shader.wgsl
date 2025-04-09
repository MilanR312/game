#import bevy_pbr::{
    pbr_fragment::pbr_input_from_standard_material,
    //parallax_mapping::parallaxed_uv,
    pbr_functions::alpha_discard,
    pbr_functions::calculate_tbn_mikktspace,
    pbr_functions::apply_normal_mapping,
    pbr_bindings,
    pbr_types,
    pbr_functions::SampleBias,
    mesh_view_bindings::view,
    //pbr_parallax_mapping::parallaxed_uv
}

#ifdef PREPASS_PIPELINE
#import bevy_pbr::{
    prepass_io::{VertexOutput, FragmentOutput},
    pbr_deferred_functions::deferred_output,
}
#else
#import bevy_pbr::{
    forward_io::{VertexOutput, FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}
#endif

struct Splat{
    splat: f32,
    index1: u32,
    index2: u32
}


fn sample_depth_map(uv: vec2<f32>, splat: Splat) -> f32 {
    // We use `textureSampleLevel` over `textureSample` because the wgpu DX12
    // backend (Fxc) panics when using "gradient instructions" inside a loop.
    // It results in the whole loop being unrolled by the shader compiler,
    // which it can't do because the upper limit of the loop in steep parallax
    // mapping is a variable set by the user.
    // The "gradient instructions" comes from `textureSample` computing MIP level
    // based on UV derivative. With `textureSampleLevel`, we provide ourselves
    // the MIP level, so no gradient instructions are used, and we can use
    // sample_depth_map in our loop.
    // See https://stackoverflow.com/questions/56581141/direct3d11-gradient-instruction-used-in-a-loop-with-varying-iteration-forcing
    return mix(
        textureSampleLevel(parallax_array, parallax_sampler, uv, splat.index1, 0.0).r,
        textureSampleLevel(parallax_array, parallax_sampler, uv, splat.index2, 0.0).r,
        splat.splat
    );
}

// An implementation of parallax mapping, see https://en.wikipedia.org/wiki/Parallax_mapping
// Code derived from: https://web.archive.org/web/20150419215321/http://sunandblackcat.com/tipFullView.php?l=eng&topicid=28
fn parallaxed_uv(
    depth_scale: f32,
    max_layer_count: f32,
    max_steps: u32,
    // The original interpolated uv
    original_uv: vec2<f32>,
    // The vector from the camera to the fragment at the surface in tangent space
    Vt: vec3<f32>,
    splat: Splat
) -> vec2<f32> {
    if max_layer_count < 1.0 {
        return original_uv;
    }
    var uv = original_uv;

    // Steep Parallax Mapping
    // ======================
    // Split the depth map into `layer_count` layers.
    // When Vt hits the surface of the mesh (excluding depth displacement),
    // if the depth is not below or on surface including depth displacement (textureSample), then
    // look forward (+= delta_uv) on depth texture according to
    // Vt and distance between hit surface and depth map surface,
    // repeat until below the surface.
    //
    // Where `layer_count` is interpolated between `1.0` and
    // `max_layer_count` according to the steepness of Vt.

    let view_steepness = abs(Vt.z);
    // We mix with minimum value 1.0 because otherwise,
    // with 0.0, we get a division by zero in surfaces parallel to viewport,
    // resulting in a singularity.
    let layer_count = mix(max_layer_count, 1.0, view_steepness);
    let layer_depth = 1.0 / layer_count;
    var delta_uv = depth_scale * layer_depth * Vt.xy * vec2(1.0, -1.0) / view_steepness;

    var current_layer_depth = 0.0;
    var texture_depth = sample_depth_map(uv, splat);

    // texture_depth > current_layer_depth means the depth map depth is deeper
    // than the depth the ray would be at this UV offset so the ray has not
    // intersected the surface
    for (var i: i32 = 0; texture_depth > current_layer_depth && i <= i32(layer_count); i++) {
        current_layer_depth += layer_depth;
        uv += delta_uv;
        texture_depth = sample_depth_map(uv, splat);
    }

#ifdef RELIEF_MAPPING
    // Relief Mapping
    // ==============
    // "Refine" the rough result from Steep Parallax Mapping
    // with a **binary search** between the layer selected by steep parallax
    // and the next one to find a point closer to the depth map surface.
    // This reduces the jaggy step artifacts from steep parallax mapping.

    delta_uv *= 0.5;
    var delta_depth = 0.5 * layer_depth;

    uv -= delta_uv;
    current_layer_depth -= delta_depth;

    for (var i: u32 = 0u; i < max_steps; i++) {
        texture_depth = sample_depth_map(uv, splat);

        // Halve the deltas for the next step
        delta_uv *= 0.5;
        delta_depth *= 0.5;

        // Step based on whether the current depth is above or below the depth map
        if (texture_depth > current_layer_depth) {
            uv += delta_uv;
            current_layer_depth += delta_depth;
        } else {
            uv -= delta_uv;
            current_layer_depth -= delta_depth;
        }
    }
#else
    // Parallax Occlusion mapping
    // ==========================
    // "Refine" Steep Parallax Mapping by interpolating between the
    // previous layer's depth and the computed layer depth.
    // Only requires a single lookup, unlike Relief Mapping, but
    // may skip small details and result in writhing material artifacts.
    let previous_uv = uv - delta_uv;
    let next_depth = texture_depth - current_layer_depth;
    let previous_depth = sample_depth_map(previous_uv, splat) - current_layer_depth + layer_depth;

    let weight = next_depth / (next_depth - previous_depth);

    uv = mix(uv, previous_uv, weight);

    current_layer_depth += mix(next_depth, previous_depth, weight);
#endif

    // Note: `current_layer_depth` is not returned, but may be useful
    // for light computation later on in future improvements of the pbr shader.
    return uv;
}




@group(2) @binding(100)
var splatmap: texture_2d<u32>;
@group(2) @binding(101)
var splatmap_sampler: sampler;
@group(2) @binding(102)
var texture_array: texture_2d_array<f32>;
@group(2) @binding(103)
var array_sampler: sampler;

@group(2) @binding(104)
var normal_array: texture_2d_array<f32>;
@group(2) @binding(105)
var normal_sampler: sampler;

@group(2) @binding(106)
var parallax_array: texture_2d_array<f32>;
@group(2) @binding(107)
var parallax_sampler: sampler;

//@group(2) @binding(108)
//var<uniform> parallax_depths: array<f32>;

// code from https://github.com/ethereumdegen/bevy_mesh_terrain/blob/main/src/shaders/terrain.wgsl
fn splat_sample(splat_uv: vec2f) -> Splat {
    let splat_map_texture_dimensions = textureDimensions(splatmap);
    let noise_distortion_amount = 2.0;

    let splat_map_sample_coord_float_1 = vec2<f32>(
      splat_uv.x * f32(splat_map_texture_dimensions.x),
      splat_uv.y * f32(splat_map_texture_dimensions.y) 
    );

    let clamped_splat_map_sample_coord_float_1 = clamp(
        splat_map_sample_coord_float_1,
        vec2f(0.0, 0.0),
        vec2<f32>(
            f32(splat_map_texture_dimensions.x - 1), 
            f32(splat_map_texture_dimensions.y - 1)
        )
    );

    let splat_map_sample_coord  = vec2<i32>(
        i32(  clamped_splat_map_sample_coord_float_1.x    ),
        i32(  clamped_splat_map_sample_coord_float_1.y   ),
    );
    let splat_index_values_at_pixel = textureLoad(splatmap, splat_map_sample_coord, 0);
    return Splat(
        f32(splat_index_values_at_pixel.b) / 255.0,
        splat_index_values_at_pixel.r,
        splat_index_values_at_pixel.g
    );
}

fn triplanar_sample(texture: texture_2d_array<f32>, x: sampler, normal: vec3<f32>, pos: vec3<f32>, idx: u32, bias: SampleBias) -> vec4<f32> {
    let blend = abs(normal);
    let sum = blend.x + blend.y + blend.z;
    let weights = blend / vec3<f32>(sum);

    let x_tex = textureSampleBias(texture, x, pos.yz, idx, bias.mip_bias);
    let y_tex = textureSampleBias(texture, x, pos.xz, idx, bias.mip_bias);
    let z_tex = textureSampleBias(texture, x, pos.xy, idx, bias.mip_bias);

    return x_tex * weights.x + y_tex * weights.y + z_tex * weights.z;
}

@fragment
fn fragment(
    in: VertexOutput,
    @builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    let double_sided = (pbr_bindings::material.flags & pbr_types::STANDARD_MATERIAL_FLAGS_DOUBLE_SIDED_BIT) != 0u;
    // TODO: mip_bias https://raw.githubusercontent.com/bevyengine/bevy/refs/tags/v0.15.3/crates/bevy_pbr/src/render/pbr_fragment.wgsl

    var bias: SampleBias;
    bias.mip_bias = view.mip_bias * 100.0;

    var pbr_input = pbr_input_from_standard_material(in, is_front);
    let parallax_splat = splat_sample(in.uv);

    let uv_transform = pbr_bindings::material.uv_transform;
    var uv = (uv_transform * vec3(in.uv, 1.0)).xy;

    let V = pbr_input.V;
    let TBN = calculate_tbn_mikktspace(in.world_normal, in.world_tangent);
    let T = TBN[0];
    let B = TBN[1];
    let N = TBN[2];
    let Vt = vec3(dot(V, T), dot(V, B), dot(V, N));
    // // TODO pass these as an array
    let scaler = mix(
        0.4,
        -0.1,
        parallax_splat.splat
    );
    let uvp_temp = parallaxed_uv(
       pbr_bindings::material.parallax_depth_scale * scaler,
       //(parallax.r) * 0.1,
       pbr_bindings::material.max_parallax_layer_count,
       pbr_bindings::material.max_relief_mapping_search_steps,
       uv,
       -Vt,
       parallax_splat
    );
    let splat = splat_sample(uvp_temp);
    let uvp = uvp_temp * 30.0 % 1;
    let pos = in.world_position.xyz * 10.0 % 1;
    //let uvp = in.uv;
    let color = mix(
        triplanar_sample(texture_array, array_sampler, pbr_input.N, pos, splat.index1, bias),
        triplanar_sample(texture_array, array_sampler, pbr_input.N, pos, splat.index2, bias),
        splat.splat
    );
    // false, true, cond
    let normal = mix(
        triplanar_sample(normal_array, normal_sampler, pbr_input.N, pos, splat.index1, bias),
        triplanar_sample(normal_array, normal_sampler, pbr_input.N, pos, splat.index2, bias),
        splat.splat
    ) * 2 - 1;
    //let normal = vec4f(0.0, 1.0, 0.0, 0.0);

    // generate a PbrInput struct from the StandardMaterial bindings

    // we can optionally modify the input before lighting and alpha_discard is applied
    pbr_input.material.base_color = color;
    //pbr_input.material.base_color = vec4(parallax.r, color.g, color.b, 1);
    pbr_input.N = normalize(pbr_input.N);
    pbr_input.clearcoat_N = pbr_input.N;
    //let TBN = calculate_tbn_mikktspace(pbr_input.world_normal, in.world_tangent);
    pbr_input.N = apply_normal_mapping(
        pbr_input.material.flags,
        TBN,
        double_sided,
        is_front,
        normal.xyz
    );
    //pbr_input.N = normal.xyz;
    //pbr_input.depth_map = depth.xyz;
    //pbr_input.parallax_depth_scale = 1.0;

    // alpha discard
    pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);

#ifdef PREPASS_PIPELINE
    // in deferred mode we can't modify anything after that, as lighting is run in a separate fullscreen shader.
    let out = deferred_output(in, pbr_input);
#else
    var out: FragmentOutput;
    // apply lighting
    out.color = apply_pbr_lighting(pbr_input);

    // we can optionally modify the lit color before post-processing is applied
    //out.color = vec4<f32>(vec4<u32>(out.color * f32(my_extended_material.quantize_steps))) / f32(my_extended_material.quantize_steps);

    // apply in-shader post processing (fog, alpha-premultiply, and also tonemapping, debanding if the camera is non-hdr)
    // note this does not include fullscreen postprocessing effects like bloom.
    out.color = main_pass_post_lighting_processing(pbr_input, out.color);

    // we can optionally modify the final result here
    //out.color = out.color * 2.0;
#endif
    return out;
}
