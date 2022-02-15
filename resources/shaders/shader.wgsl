// Vertex

struct VertexInput {
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] color: vec4<f32>;
    [[location(2)]] uv: vec2<f32>;
};

struct InstanceInput {
    [[location(5)]] matrix_0: vec4<f32>;
    [[location(6)]] matrix_1: vec4<f32>;
    [[location(7)]] matrix_2: vec4<f32>;
    [[location(8)]] matrix_3: vec4<f32>;
};

struct Uniform {
    mvp: mat4x4<f32>;
};
[[group(1), binding(0)]]
var<uniform> uni: Uniform;

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec4<f32>;
    [[location(1)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn vs_main(
    model: VertexInput,
    instance: InstanceInput,
) -> VertexOutput {
    let instance_matrix = mat4x4<f32>(
        instance.matrix_0,
        instance.matrix_1,
        instance.matrix_2,
        instance.matrix_3,
    );

    var out: VertexOutput;
    out.color = model.color;
    out.uv = model.uv;
    let position = instance_matrix * vec4<f32>(model.position, 0.0, 1.0);
    let out_pos = uni.mvp * position;
    out.clip_position = out_pos;
    return out;
}

// Fragment

[[group(0), binding(0)]]
var t_diffuse: texture_2d<f32>;
[[group(0), binding(1)]]
var s_diffuse: sampler;

[[stage(fragment)]]
fn fs_main(in: VertexOutput) -> [[location(0)]] vec4<f32> {
    return in.color * textureSample(t_diffuse, s_diffuse, in.uv);
    //return textureSample(t_diffuse, s_diffuse, in.uv);
    //return vec4<f32>(in.color, 1.0);
}