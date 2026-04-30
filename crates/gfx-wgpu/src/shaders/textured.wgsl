// textured.wgsl — textured quad (same Uniforms layout as fill.wgsl)

struct Uniforms {
    transform_col0 : vec4<f32>,
    transform_col1 : vec4<f32>,
    transform_col2 : vec4<f32>,
    tint           : vec4<f32>,
}

@group(0) @binding(0)
var<uniform> u: Uniforms;

@group(0) @binding(1)
var tex: texture_2d<f32>;

@group(0) @binding(2)
var samp: sampler;

struct VertexInput {
    @location(0) position : vec2<f32>,
    @location(1) uv       : vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_pos : vec4<f32>,
    @location(0)       uv       : vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    let t = mat3x3<f32>(
        u.transform_col0.xyz,
        u.transform_col1.xyz,
        u.transform_col2.xyz,
    );
    let p = t * vec3<f32>(in.position, 1.0);
    var out: VertexOutput;
    out.clip_pos = vec4<f32>(p.xy, 0.0, 1.0);
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let c = textureSample(tex, samp, in.uv);
    return c * u.tint;
}
