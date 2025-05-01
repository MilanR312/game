#import bevy_pbr::{
    mesh_view_bindings::globals,
    skinning,
    pbr_types::pbr_input_new,
    pbr_functions::alpha_discard,
    
    mesh_view_bindings::view,
    mesh_functions,
    morph::morph,
    view_transformations::position_world_to_clip
}
#ifdef PREPASS_PIPELINE
error
#endif

#import bevy_pbr::{
    forward_io::{FragmentOutput},
    pbr_functions::{apply_pbr_lighting, main_pass_post_lighting_processing},
}



struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) normal: vec3<f32>,

    //@location(3) segment_distance: f32,
    //@location(4) segment_length: f32,
}
struct VertexOutput {
    // This is `clip position` when the struct is used as a vertex stage output
    // and `frag coord` when used as a fragment stage input
    @builtin(position) position: vec4<f32>,
    @location(0) world_position: vec4<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    //@location(3) segment_distance: f32,
    //@location(4) segment_length: f32,
}



@vertex
fn vertex(vertex_no_morph: Vertex) -> VertexOutput {
    var out: VertexOutput;
    //out.segment_distance = vertex_no_morph.segment_distance;
    //out.segment_length = vertex_no_morph.segment_length;

#ifdef MORPH_TARGETS
    var vertex = morph_vertex(vertex_no_morph);
#else
    var vertex = vertex_no_morph;
#endif

#ifdef SKINNED
    var model = skinning::skin_model(vertex.joint_indices, vertex.joint_weights);
#else
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416 .
    var model = mesh_functions::get_world_from_local(vertex_no_morph.instance_index);
    //var model = mesh_functions::get_model_matrix(vertex_no_morph.instance_index);
#endif


#ifdef VERTEX_NORMALS
#ifdef SKINNED
    out.world_normal = skinning::skin_normals(model, vertex.normal);
#else
    out.world_normal = mesh_functions::mesh_normal_local_to_world(
        vertex.normal,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif
#endif

#ifdef VERTEX_POSITIONS
    out.world_position = mesh_functions::mesh_position_local_to_world(model, vec4<f32>(vertex.position, 1.0));
    out.position = position_world_to_clip(out.world_position.xyz);
#endif

#ifdef VERTEX_UVS
    out.uv = vertex.uv;
#endif

#ifdef VERTEX_UVS_B
    out.uv_b = vertex.uv_b;
#endif

#ifdef VERTEX_TANGENTS
    out.world_tangent = mesh_functions::mesh_tangent_local_to_world(
        model,
        vertex.tangent,
        // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
        // See https://github.com/gfx-rs/naga/issues/2416
        vertex_no_morph.instance_index
    );
#endif

#ifdef VERTEX_COLORS
    out.color = vertex.color;
#endif

#ifdef VERTEX_OUTPUT_INSTANCE_INDEX
    // Use vertex_no_morph.instance_index instead of vertex.instance_index to work around a wgpu dx12 bug.
    // See https://github.com/gfx-rs/naga/issues/2416
    //out.instance_index = vertex_no_morph.instance_index;
#endif

    return out;
}

@group(2) @binding(0) var<uniform> length: f32;
@group(2) @binding(1) var<uniform> is_disabled: u32;



fn permute_four(x: vec4<f32>) -> vec4<f32> { return ((x * 34. + 1.) * x) % vec4<f32>(289.); }
fn fade_two(t: vec2<f32>) -> vec2<f32> { return t * t * t * (t * (t * 6. - 15.) + 10.); }

fn perlin_noise_2d(P: vec2<f32>) -> f32 {
  var Pi: vec4<f32> = floor(P.xyxy) + vec4<f32>(0., 0., 1., 1.);
  let Pf = fract(P.xyxy) - vec4<f32>(0., 0., 1., 1.);
  Pi = Pi % vec4<f32>(289.); // To avoid truncation effects in permutation
  let ix = Pi.xzxz;
  let iy = Pi.yyww;
  let fx = Pf.xzxz;
  let fy = Pf.yyww;
  let i = permute_four(permute_four(ix) + iy);
  var gx: vec4<f32> = 2. * fract(i * 0.0243902439) - 1.; // 1/41 = 0.024...
  let gy = abs(gx) - 0.5;
  let tx = floor(gx + 0.5);
  gx = gx - tx;
  var g00: vec2<f32> = vec2<f32>(gx.x, gy.x);
  var g10: vec2<f32> = vec2<f32>(gx.y, gy.y);
  var g01: vec2<f32> = vec2<f32>(gx.z, gy.z);
  var g11: vec2<f32> = vec2<f32>(gx.w, gy.w);
  let norm = 1.79284291400159 - 0.85373472095314 *
      vec4<f32>(dot(g00, g00), dot(g01, g01), dot(g10, g10), dot(g11, g11));
  g00 = g00 * norm.x;
  g01 = g01 * norm.y;
  g10 = g10 * norm.z;
  g11 = g11 * norm.w;
  let n00 = dot(g00, vec2<f32>(fx.x, fy.x));
  let n10 = dot(g10, vec2<f32>(fx.y, fy.y));
  let n01 = dot(g01, vec2<f32>(fx.z, fy.z));
  let n11 = dot(g11, vec2<f32>(fx.w, fy.w));
  let fade_xy = fade_two(Pf.xy);
  let n_x = mix(vec2<f32>(n00, n01), vec2<f32>(n10, n11), vec2<f32>(fade_xy.x));
  let n_xy = mix(n_x.x, n_x.y, fade_xy.y);
  return 2.3 * n_xy;
}


fn noise_at_point(p: vec2f) -> f32 {
    let y_scaler = 3.0;
    let x_scaler = 3.0;
    let side_time_scaler = 0.3;

    let transform_time = globals.time * side_time_scaler;
    let upper_offset = ceil(transform_time);
    let lower_offset = floor(transform_time);

    let upper_noise = perlin_noise_2d(vec2f(p.y * y_scaler, (p.x + upper_offset) * x_scaler));
    let lower_noise = perlin_noise_2d(vec2f(p.y * y_scaler, (p.x + lower_offset) * x_scaler));
    let factor = fract(transform_time);
    let noise = mix(
        lower_noise,
        upper_noise,
        factor
    );
    return noise;
}
fn color_at_point(p: vec2f) -> vec4f {
    let noise = noise_at_point(p);
    let alpha_mask = select(
        0.0,
        1.0,
        noise > 0.2
    );

    let max_color = vec3f(63.0 / 255.0, 171.0 / 255.0, 159.0 / 255.0);
    let min_color = vec3f(22.0 / 255.0, 95.0 / 255.0, 76.0 / 255.0);

    var color = mix(
        min_color,
        max_color,
        noise
    );
    let light_strength = cos(abs(p.y * 3.1415)) * 0.25 + 0.5;
    color = color * light_strength;
    

    return vec4f(
        color,
        alpha_mask
    ) * 3.0;
}

fn blurred_color_at_point(p: vec2f) -> vec4f {
    var main_color = color_at_point(p);
    let side_blender = abs(p.y);
    main_color.a = cos(side_blender * 3.1415) * 0.5 + 0.5;

    let dx = 0.4;
    let color_left = color_at_point(vec2f(p.x - dx, p.y));
    let color_right = color_at_point(vec2f(p.x + dx, p.y));
    var blended_color = mix(
        color_left,
        color_left,
        0.5
    );
    blended_color.a = 0.5;
    let out_color = select(
        blended_color,
        main_color,
        main_color.a != 0.0
    );
    let alpha_mask = select(
        out_color.a,
        0.0,
        is_disabled == 1
    );
    return vec4f(out_color.rgb, alpha_mask);
}

@fragment
fn fragment(
    in: VertexOutput,
    //@builtin(front_facing) is_front: bool,
) -> FragmentOutput {
    // generate a PbrInput struct from the StandardMaterial bindings
    var pbr_input = pbr_input_new();
    //let is_first_segment = sign(in.segmenth_length) == 1.0;
    //let segment_length = abs(in.segmenth_length);
    //let uvs = select(
    //    (1.0 - in.uv),
    //    in.uv,
    //    vec2(false)
    //)  * length % 1.0;
    //let uvs = (1.0 - in.uv) * length / 5.0 % 1.0;
    var curve_local_uvs = 1.0 - select(
        1.0 - (-in.uv.y) % 1.0,
        in.uv.y % 1.0,
        in.uv.y > 0.0
    );
    curve_local_uvs = 1.0 - select(
        1.0 - (-in.uv.y),
        in.uv.y,
        in.uv.y > 0.0
    );
    let time_scaler = 0.2;
    
    var global_uvs = in.uv.y - length;
    global_uvs = in.uv.y;
    global_uvs = (global_uvs - time_scaler * globals.time);

    let noise_pos = vec2f(global_uvs, in.uv.x);
    let noise = noise_at_point(noise_pos);


    
    let color = blurred_color_at_point(noise_pos);
    
    pbr_input.material.base_color = color;
    
    //pbr_input.material.base_color = vec4f(
    //    curve_local_uvs,
    //    0.0,
    //    0.0,
    //    1.0
    //);


    //curve_local_uvs = (curve_local_uvs + 0.5 * globals.time);
    //curve_local_uvs = (curve_local_uvs + 1.0) % 1.0;
    //curve_local_uvs = curve_local_uvs / max(in.segment_length, 0.1);
    //curve_local_uvs = (curve_local_uvs * in.segment_length) % 1.0;
    //let curve_local_uvs = (in.uv.y - in.segmenth_length);
    //let curve_local_uvs = in.uv * in.segmenth_length;
    //let uvs = (1.0-in.uv) * in.segmenth_length * 5.0 % 1.0;
    //let uvs = (1.0 - in.uv) * in.segmenth_length % 1.0;
    //let uvs = 1.0 - uvs1;
    // we can optionally modify the input before lighting and alpha_discard is applied
    //pbr_input.material.base_color.b = pbr_input.material.base_color.r;

    // alpha discard
    //pbr_input.material.base_color = alpha_discard(pbr_input.material, pbr_input.material.base_color);
    //pbr_input.material.base_color.a = 0.0;

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

    //out.color = vec4f(0.0, foo, 0.0, alpha_mask);
    return out;
}
