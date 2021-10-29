// Vertex

struct VertexInput {
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] color: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

struct InstanceInput {
    [[location(3)]] position: vec2<f32>;
    [[location(4)]] scale: vec2<f32>;
    [[location(5)]] rotation: f32;
};

[[block]]
struct Uniform {
    position: vec2<f32>;
    scale: vec2<f32>;
    rotation: f32;
};
[[group(1), binding(0)]]
var<uniform> uniform: Uniform;

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec3<f32>;
    [[location(1)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color;
    out.uv = model.uv;

    // TODO help
    let norm_pos = model.position * vec2<f32>(2.0, 2.0) - vec2<f32>(1.0, 1.0);
    let total_scale = uniform.scale * instance.scale;
    let total_rotation = -(uniform.rotation + instance.rotation);
    var total_translation = uniform.position + instance.position;
    total_translation.y = -total_translation.y; // wgpu is y+ up while we want y+ down

    let scaled_pos = norm_pos * total_scale;
    let rotated_pos = vec2<f32>(
        cos(total_rotation) * scaled_pos.x - sin(total_rotation) * scaled_pos.y,
        sin(total_rotation) * scaled_pos.x + cos(total_rotation) * scaled_pos.y,
    );
    let top_lefted_pos = rotated_pos + vec2<f32>(-1.0, 1.0);
    let translated_pos = top_lefted_pos + total_translation * 2.0;

    out.clip_position = vec4<f32>(translated_pos, 0.0, 1.0);
    return out;
}

// Fragment

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;

[[stage(fragment)]]
fn main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return vec4<f32>(in.color, 1.0) * textureSample(t_diffuse, s_diffuse, in.uv);
    //return textureSample(t_diffuse, s_diffuse, in.uv);
    //return vec4<f32>(in.color, 1.0);
}