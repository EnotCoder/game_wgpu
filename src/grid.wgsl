struct Uniforms {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
};

@vertex
fn vs_main(@location(0) position: vec3<f32>) -> VertexOutput {
    var out: VertexOutput;
    out.position = uniforms.view_proj * vec4<f32>(position, 1.0);
    out.world_pos = position;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let wx = in.world_pos.x;
    let wz = in.world_pos.z;
    let dist = length(vec2<f32>(wx, wz));

    let gx = wx - round(wx);
    let gz = wz - round(wz);
    let minor_dist = min(abs(gx), abs(gz));
    let minor_line = 1.0 - smoothstep(0.0, 0.012, minor_dist);

    let mx = wx / 10.0 - round(wx / 10.0);
    let mz = wz / 10.0 - round(wz / 10.0);
    let major_dist = min(abs(mx * 10.0), abs(mz * 10.0));
    let major_line = 1.0 - smoothstep(0.0, 0.025, major_dist);

    let is_axis = abs(wx) < 0.015 || abs(wz) < 0.015;
    let axis_line = select(0.0, 1.0, is_axis);

    let line = max(max(minor_line * 0.4, major_line * 0.7), axis_line * 0.9);

    let fade = 1.0 - smoothstep(5.0, 40.0, dist);
    let alpha = line * fade;

    let color = select(
        vec3<f32>(0.45, 0.45, 0.5),
        vec3<f32>(0.55, 0.55, 0.65),
        is_axis,
    );

    return vec4<f32>(color, alpha);
}
