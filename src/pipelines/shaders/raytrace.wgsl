@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;

@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let screen_size: vec2<i32> = vec2<i32>(textureDimensions(color_buffer));
    let screen_pos: vec2<i32> = vec2<i32>(i32(GlobalInvocationID.x), i32(GlobalInvocationID.y));

    if screen_pos.x >= screen_size.x || screen_pos.y >= screen_size.y {
        return;
    }

    let fx: f32 = f32(screen_pos.x);
    let fy: f32 = f32(screen_pos.y);

    if(screen_pos.x >= 50 && screen_pos.x <= 60) {
        return;
    }
    textureStore(color_buffer, screen_pos, vec4<f32>(fx / 500.0, fy / 500.0, 1.0, 1.0));
}
