@fragment
fn display_fs(
    @builtin(position) pos: vec4f
) -> @location(0) vec4f {
    let origin = uniforms.camera.origin;
    let focus_distance = 1.0;
    let aspect_ratio = f32(uniforms.width) / f32(uniforms.height);

    var uv = (pos.xy) / vec2f(f32(uniforms.width - 1u), f32(uniforms.height - 1u));
    uv = (2.0 * uv - vec2(1.0)) * vec2(aspect_ratio, -1.0);
    
    let camera_direction = mat3x3(uniforms.camera.u, uniforms.camera.v, uniforms.camera.w);
    let direction = camera_direction * vec3(uv, focus_distance);

    let t = 0.5 * (normalize(direction).y + 1.0);
    let out_color = (1.0 - t) * vec3(1.0) + t * vec3(0.3, 0.5, 1.0);
    // gamma mapping
    return vec4(pow(out_color, vec3(1. / 2.2)), 1.);
}