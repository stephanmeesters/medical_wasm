// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
};

@vertex
fn vs_main(model: VertexInput) -> VertexOutput {
    let mvp = mat4x4<f32>(
        vec4<f32>(0.5, 0.0, 0.0, 0.0),
        vec4<f32>(0.0, 0.5, 0.0, 0.0),
        vec4<f32>(0.0, 0.0, 0.5, 0.0),
        vec4<f32>(0.0, 0.0, 0.0, 1.0),
    );
    // let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    // let ambient = 0.1;
    // let diffuse = max(dot(input.normal, light_dir), 0.0);
    // let color = vec3<f32>(1.0, 1.0, 1.0) * (ambient + diffuse);

    var out: VertexOutput;
    out.clip_position = mvp * vec4<f32>(model.position, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let pp = in.clip_position.z*in.clip_position.z;
    return vec4<f32>(pp, pp, pp, 1.0);
}
