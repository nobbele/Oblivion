// Vertex

struct VertexInput {
    [[location(0)]] position: vec2<f32>;
    [[location(1)]] color: vec3<f32>;
    [[location(2)]] uv: vec2<f32>;
};

struct VertexOutput {
    [[builtin(position)]] clip_position: vec4<f32>;
    [[location(0)]] color: vec3<f32>;
    [[location(1)]] uv: vec2<f32>;
};

[[stage(vertex)]]
fn main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    out.color = model.color * vec3<f32>(1.0, 0.0, 0.0);
    out.uv = model.uv;
    out.clip_position = vec4<f32>(model.position, 0.0, 1.0);
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
}