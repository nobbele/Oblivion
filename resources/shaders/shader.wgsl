// Vertex

struct VertexInput {
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] color: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

struct InstanceInput {
    [[location(5)]] matrix_0: vec4<f32>;
    [[location(6)]] matrix_1: vec4<f32>;
    [[location(7)]] matrix_2: vec4<f32>;
    [[location(8)]] matrix_3: vec4<f32>;
};

[[block]]
struct Uniform {
    mvp: mat4x4<f32>;
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
    let instance_matrix = mat4x4<f32>(
        instance.matrix_0,
        instance.matrix_1,
        instance.matrix_2,
        instance.matrix_3,
    );

    var out: VertexOutput;
    out.color = model.color;
    out.uv = model.uv;
    var instance_position = (instance_matrix * vec4<f32>(model.position, 0.0, 1.0)).xy;
    var target_position = (uniform.mvp * vec4<f32>(instance_position, 0.0, 1.0)).xy;
    target_position.y = 2.0 - target_position.y;
    out.clip_position = vec4<f32>(target_position - vec2<f32>(1.0, 1.0), 0.0, 1.0);
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