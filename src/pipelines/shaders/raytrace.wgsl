struct CameraUniform {
    view_proj: mat4x4<f32>,
};

struct Ray {
    start: vec3<f32>,
    direction: vec3<f32>
}

@group(0) @binding(0) var color_buffer: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1) var<uniform> camera: CameraUniform;

const fov: f32 = 1.6;
const screen_width: i32 = 500;
const screen_height: i32 = 500;

const polygon_positions = array<vec3<f32>, 3>(
    vec3<f32>(-0.0, -0.0, 0.0),
    vec3<f32>(-0.5, -0.0, 0.0),
    vec3<f32>(0.0, -0.5, 0.0),
);

@compute @workgroup_size(8,8,1)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let screen_size: vec2<i32> = vec2<i32>(textureDimensions(color_buffer));
    let screen_pos: vec2<i32> = vec2<i32>(i32(GlobalInvocationID.x), i32(GlobalInvocationID.y));

    if screen_pos.x >= screen_size.x || screen_pos.y >= screen_size.y {
        return;
    }

    // let fx: f32 = f32(screen_pos.x);
    // let fy: f32 = f32(screen_pos.y);
    //
    // if(screen_pos.x >= 50 && screen_pos.x <= 60) {
    //     return;
    // }
    // textureStore(color_buffer, screen_pos, vec4<f32>(fx / 500.0, fy / 500.0, 1.0, 1.0));

//     let npos1 = camera.view_proj * vec4<f32>(polygon_positions[0], 1.0);
//     let npos2 = camera.view_proj * vec4<f32>(polygon_positions[1], 1.0);
//     let npos3 = camera.view_proj * vec4<f32>(polygon_positions[2], 1.0);
//     let npos = array<vec3<f32>,3>(
//     npos1.xyz,
//     npos2.xyz,
//     npos3.xyz,
// );

    let camera_pos = camera.view_proj * vec4<f32>(0.0, 0.0, 1.0, 1.0);

    var ray = raystart(screen_pos, camera_pos.xyz, camera.view_proj);
    // var weights = polygon_ray_intersection(ray, polygon_positions);

    if PointInTriangle(ray, polygon_positions) {
        var weights = PointInTriangleCoords(ray, polygon_positions);
        textureStore(color_buffer, screen_pos, vec4<f32>(weights, 1.0));
    } else {
        textureStore(color_buffer, screen_pos, vec4<f32>(0.0));
    }


    // textureStore(color_buffer, screen_pos, vec4<f32>(weights[0] + weights[1] + weights[2]));

    // if weights[0] < 0.0 || weights[1] < 0.0 || weights[2] < 0.0 {
    // if weights[0] + weights[1] + weights[2] < 1.0 {
    //     textureStore(color_buffer, screen_pos, vec4<f32>(1.0));
    // } else {
    //     textureStore(color_buffer, screen_pos, vec4<f32>(0.0));
    // }
    //
    // var bb = ray.direction;
    // bb += vec3<f32>(1.0);
    // bb /= 5.0;

    // if(ray.start.x > -0.5 && ray.start.x < 0.5 &&
    //     ray.start.y > -0.5 && ray.start.y < 0.5)
    // {
    //     textureStore(color_buffer, screen_pos, vec4<f32>(1.0));
    // }
    // else
    // {
    //     textureStore(color_buffer, screen_pos, vec4<f32>(0.0));
    // }
}

fn sign(p1: vec3<f32>, p2: vec3<f32>, p3: vec3<f32>) -> f32 {
    return (p1.x - p3.x) * (p2.y - p3.y) - (p2.x - p3.x) * (p1.y - p3.y);
}

fn PointInTriangle(pt: Ray, polygon: array<vec3<f32>, 3>) -> bool {
    let v1 = polygon[0];
    let v2 = polygon[1];
    let v3 = polygon[2];

    let d1 = sign(pt.start, v1, v2);
    let d2 = sign(pt.start, v2, v3);
    let d3 = sign(pt.start, v3, v1);

    let has_neg = (d1 < 0) || (d2 < 0) || (d3 < 0);
    let has_pos = (d1 > 0) || (d2 > 0) || (d3 > 0);

    return !(has_neg && has_pos);
}

fn PointInTriangleCoords(pt: Ray, polygon: array<vec3<f32>, 3>) -> vec3<f32> {
    let v1 = polygon[0];
    let v2 = polygon[1];
    let v3 = polygon[2];

    let d1 = sign(pt.start, v1, v2);
    let d2 = sign(pt.start, v2, v3);
    let d3 = sign(pt.start, v3, v1);

    return vec3<f32>(d1, d2, d3);
}

fn raystart(screenPos: vec2<i32>, camPos: vec3<f32>, trans: mat4x4<f32>) -> Ray {
    let s = -2.0 * tan(fov * 0.5);
    let start = vec3<f32>(
        (f32(screenPos.x) / f32(screen_width) - 0.5f) * s,
        -(f32(screenPos.y) / f32(screen_height) - 0.5f) * s,
        1.0
    );

    let ttt = (trans * vec4<f32>(start, 1.0)).xyz;
    return Ray(ttt , normalize(ttt - camPos));
}

// compute barycentic coordinates
fn polygon_ray_intersection(ray: Ray, polygon: array<vec3<f32>,3>) -> vec3<f32> {
    let e1 = polygon[1] - polygon[0];
    let e2 = polygon[2] - polygon[0];
    let q = cross(ray.direction, e2);

    let a = dot(e1, q);

    let s = ray.start - polygon[0];
    let r = cross(s, e1);

    var weight = vec3<f32>(0.0);
    weight[0] = dot(s, q) / a;
    weight[1] = dot(ray.direction, r) / a;
    weight[2] = 1.0 - (weight[1] + weight[2]);

    return weight;
}
