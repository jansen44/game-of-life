struct VertexInput {
    @location(0) pos: vec2<f32>,
};

struct InstanceInput {
    @location(1) model_matrix_0: vec4<f32>,
    @location(2) model_matrix_1: vec4<f32>,
    @location(3) model_matrix_2: vec4<f32>,
    @location(4) model_matrix_3: vec4<f32>,
    @location(5) state: u32,
}

@group(0) @binding(0)
var<uniform> ortho_proj: mat4x4<f32>;

struct VertexOutput {
    @builtin(position) pos: vec4<f32>,
    @location(0) state: u32,
};

@vertex
fn vs_main(input: VertexInput, instance: InstanceInput) -> VertexOutput {
    let model = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );

    var out: VertexOutput;
    out.pos = ortho_proj * model * vec4<f32>(input.pos.xy, 1.0, 1.0);
    out.state = instance.state;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    if (in.state == u32(1)) {
        return vec4<f32>(0.6, 0.7, 0.8, 1.0);
    }
    return vec4<f32>(0.07, 0.07, 0.09, 1.0);
}
