@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> @builtin(position) vec4<f32> {
    var pos: array<vec2<f32>, 6>;
    pos[0] = vec2<f32>(-1.0,  1.0);
    pos[1] = vec2<f32>( 1.0,  1.0);
    pos[2] = vec2<f32>( 1.0, -1.0);

    pos[3] = vec2<f32>(-1.0,  1.0);
    pos[4] = vec2<f32>(-1.0, -1.0);
    pos[5] = vec2<f32>( 1.0, -1.0);

    return vec4<f32>(pos[in_vertex_index], 0.0, 1.0);
}

@fragment
fn fs_main() -> @location(0) vec4<u32> {
    return vec4<u32>(u32(0));
}
