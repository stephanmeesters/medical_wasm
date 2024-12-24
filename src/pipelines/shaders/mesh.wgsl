// Vertex shader

struct CameraUniform {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    direction: vec3<f32>
};
@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>
};

struct VertexOutput {
    @location(0) normal: vec3<f32>,
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = camera.view_proj * vec4<f32>(model.position, 1.0);
    out.normal = model.normal;
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // let pp = length(in.normal.z);
    let light = vec3(0.0, 1.0, 0.0);
    let pp = dot(in.normal, light); 
    return vec4<f32>(pp, pp, pp, 1.0);
}
