// fill.wgsl — solid-colour fill shader
//
// Uniforms (group 0, binding 0):
//   The 2D affine transform (Mat3) is passed as three vec4 columns to avoid
//   the std140 padding issue with mat3x3<f32> (each column would be padded
//   to 16 bytes anyway; we make that explicit on the Rust side too).
//   `tint` is multiplied with the per-vertex colour.

struct Uniforms {
    transform_col0 : vec4<f32>,
    transform_col1 : vec4<f32>,
    transform_col2 : vec4<f32>,
    tint           : vec4<f32>,
}

@group(0) @binding(0)
var<uniform> u: Uniforms;

// ---- vertex stage ----

struct VertexInput {
    @location(0) position : vec2<f32>,
    @location(1) color    : vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_pos : vec4<f32>,
    @location(0)       color    : vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    // Reconstruct the mat3x3 from the padded columns.
    let t = mat3x3<f32>(
        u.transform_col0.xyz,
        u.transform_col1.xyz,
        u.transform_col2.xyz,
    );

    // Apply the 2D affine transform (homogeneous: z = 1).
    let p = t * vec3<f32>(in.position, 1.0);

    var out: VertexOutput;
    out.clip_pos = vec4<f32>(p.xy, 0.0, 1.0);
    out.color    = in.color * u.tint;
    return out;
}

// ---- fragment stage ----

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.color;
}
