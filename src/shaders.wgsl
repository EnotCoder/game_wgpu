struct Uniforms {
    translation: vec4<f32>,
    rotation: vec4<f32>,
    view_proj: mat4x4<f32>,
    use_texture: i32,
    base_color: vec4<f32>,
    light_dir: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var my_texture: texture_2d<f32>;

@group(1) @binding(1)
var my_sampler: sampler;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coord: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
};

fn rotate_x(vertex: vec3<f32>, angle: f32) -> vec3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec3<f32>(
        vertex.x,
        vertex.y * c - vertex.z * s,
        vertex.y * s + vertex.z * c
    );
}

fn rotate_y(vertex: vec3<f32>, angle: f32) -> vec3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec3<f32>(
        vertex.x * c + vertex.z * s,
        vertex.y,
        -vertex.x * s + vertex.z * c
    );
}

fn rotate_z(vertex: vec3<f32>, angle: f32) -> vec3<f32> {
    let c = cos(angle);
    let s = sin(angle);
    return vec3<f32>(
        vertex.x * c - vertex.y * s,
        vertex.x * s + vertex.y * c,
        vertex.z
    );
}

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) tex_coord: vec2<f32>,
    @location(2) normal: vec3<f32>,
) -> VertexOutput {
    var output: VertexOutput;

    var rotated = position;
    rotated = rotate_x(rotated, uniforms.rotation.x);
    rotated = rotate_y(rotated, uniforms.rotation.y);
    rotated = rotate_z(rotated, uniforms.rotation.z);

    var rotated_normal = normal;
    rotated_normal = rotate_x(rotated_normal, uniforms.rotation.x);
    rotated_normal = rotate_y(rotated_normal, uniforms.rotation.y);
    rotated_normal = rotate_z(rotated_normal, uniforms.rotation.z);

    let world_pos = rotated + uniforms.translation.xyz;

    output.position = uniforms.view_proj * vec4<f32>(world_pos, 1.0);
    output.tex_coord = tex_coord;
    output.world_normal = rotated_normal;

    return output;
}

@fragment
fn fs_main(
    @location(0) tex_coord: vec2<f32>,
    @location(1) world_normal: vec3<f32>,
) -> @location(0) vec4<f32> {
    let light_dir = normalize(uniforms.light_dir.xyz);
    let diffuse = max(dot(normalize(world_normal), light_dir), 0.0);
    let ambient = 0.2;
    let lighting = diffuse + ambient;

    if uniforms.use_texture == 1 {
        let tex_color = textureSample(my_texture, my_sampler, tex_coord);
        return vec4<f32>(tex_color.rgb * lighting, tex_color.a);
    } else {
        return vec4<f32>(uniforms.base_color.rgb * lighting, uniforms.base_color.a);
    }
}
