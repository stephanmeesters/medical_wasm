struct CameraUniform {
    // Camera parameters
    position: vec3<f32>,
    _pad1: f32,
    direction: vec3<f32>,
    _pad2: f32,
    up: vec3<f32>,
    _pad3: f32,
    side: vec3<f32>,
    _pad4: f32,
};

struct Ray {
    start: vec3<f32>,
    direction: vec3<f32>,
}

struct Hit {
    hit: bool,
    t: f32,
    p: vec3<f32>,
    normal: vec3<f32>
}

@group(0) @binding(0)
var color_buffer: texture_storage_2d<rgba8unorm, write>;
@group(0) @binding(1)
var<uniform> camera: CameraUniform;

const fov: f32 = 1;
const screen_width: f32 = 1000.0;
const screen_height: f32 = 1000.0;

const polygon_positions = array<vec3<f32>, 3>(
    vec3<f32>(-0.0, -0.0, 0.0),
    vec3<f32>(-0.5, -0.0, 0.0),
    vec3<f32>(0.0, -0.5, 0.0),
);

const numSamples: u32 = 100;
const useAA: bool = true;

fn hash22(p: vec2<f32>) -> vec2<f32> {
    var p3 = fract(vec3<f32>(p.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

fn hash32(p: vec2<f32>) -> vec3<f32> {
    var p3 = fract(vec3<f32>(p.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yxz + 33.33);
    return fract((p3.xxy + p3.yzz) * p3.zyx);
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let screen_size: vec2<u32> = textureDimensions(color_buffer);
    let screen_pos_u32: vec2<u32> = GlobalInvocationID.xy;
    let global_id_1d: u32 = screen_pos_u32.y * screen_size.x + screen_pos_u32.x;

    if screen_pos_u32.x >= screen_size.x || screen_pos_u32.y >= screen_size.y {
        return;
    }

    if useAA {
        var color: vec4<f32>;
        let offset = 1.0;
        for (var i: u32 = 0; i < numSamples; i = i + 1) {
            let coord = hash22(vec2<f32>(f32(screen_pos_u32.x) * 10000 + f32(i) * 123123, f32(screen_pos_u32.y) * 100000 + f32(i) * 456456)) - vec2<f32>(0.5);
            color += castray(vec2<f32>(f32(screen_pos_u32.x) + coord.x * offset, f32(screen_pos_u32.y) + coord.y * offset));
        }
        color /= f32(numSamples);
        let screen_pos_i32 = vec2<i32>(screen_pos_u32);
        textureStore(color_buffer, screen_pos_i32, color);
    } else {
        let offset = 0.5;
        var color = castray(vec2<f32>(screen_pos_u32));
        let screen_pos_i32 = vec2<i32>(screen_pos_u32);
        textureStore(color_buffer, screen_pos_i32, color);
    }
}

fn castray(screen_pos: vec2<f32>) -> vec4<f32> {
    var ray = raystart(screen_pos);

    let sphere_pos = vec3<f32>(0.0, 0.0, 0.0);
    let sphere_pos_2 = vec3<f32>(0.0, 0.0, -8.0);
    let sphere_radius = 0.3;
    let sphere_radius_2 = 0.8;


    var hit_anything = false;
    var closest = Hit(false, 0.0, vec3<f32>(0.0), vec3<f32>(0.0));

    let hit_sphere = hit_sphere(sphere_pos, sphere_radius, ray, 1.0, 1000.0);
    if hit_sphere.hit {
        closest = hit_sphere;
        hit_anything = true;
    }

    let hit_sphere_2 = hit_sphere(sphere_pos_2, sphere_radius_2, ray, 1.0, 1000.0);
    if hit_sphere_2.hit && (!hit_anything || hit_sphere_2.t < closest.t) {
        closest = hit_sphere_2;
        hit_anything = true;
    }

    if hit_anything {
        return vec4<f32>((closest.normal + 1.0) / 2.0, 1.0);
    }
    
    return vec4<f32>(0.0, 0.0, 0.0, 1.0);
}

fn hit_sphere(center: vec3<f32>, radius: f32, r: Ray, near: f32, far: f32) -> Hit {
    let oc = center - r.start;
    let a = dot(r.direction, r.direction);
    let h = dot(oc, r.direction);
    let c = dot(oc, oc) - radius * radius;
    let discriminant = h * h - a * c;

    if discriminant < 0.0 {
        return Hit(false, 0.0, vec3<f32>(0.0), vec3<f32>(0.0));
    }

    let sqrtd = sqrt(discriminant);
    var root = (h - sqrtd) / a;

    if root <= near || root >= far {
        root = (h + sqrtd) / a;
        if root <= near || root >= far {
            return Hit(false, 0.0, vec3<f32>(0.0), vec3<f32>(0.0));
        }
    }

    let p = r.start + r.direction * root;
    let normal = normalize(p - center);

    return Hit(true, root, p, normal);
}

fn intersectTriangle(ray: Ray, v0: vec3<f32>, v1: vec3<f32>, v2: vec3<f32>) -> f32 {
    let edge1 = v1 - v0;
    let edge2 = v2 - v0;
    let h = cross(ray.direction, edge2);
    let a = dot(edge1, h);
    if abs(a) < 1e-5 {
        return -1.0; // Ray is parallel to triangle
    }
    let f = 1.0 / a;
    let s = ray.start - v0;
    let u = f * dot(s, h);
    if u < 0.0 || u > 1.0 {
        return -1.0;
    }
    let q = cross(s, edge1);
    let v = f * dot(ray.direction, q);
    if v < 0.0 || (u + v) > 1.0 {
        return -1.0;
    }
    let t = f * dot(edge2, q);
    if t > 1e-5 {
        return t; // Intersection with the triangle
    } else {
        return -1.0; // No valid intersection
    }
}

fn computeBarycentricCoords(p: vec3<f32>, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>) -> vec3<f32> {
    let v0 = b - a;
    let v1 = c - a;
    let v2 = p - a;
    let d00 = dot(v0, v0);
    let d01 = dot(v0, v1);
    let d11 = dot(v1, v1);
    let d20 = dot(v2, v0);
    let d21 = dot(v2, v1);
    let denom = d00 * d11 - d01 * d01;
    let v = (d11 * d20 - d01 * d21) / denom;
    let w = (d00 * d21 - d01 * d20) / denom;
    let u = 1.0 - v - w;
    return vec3<f32>(u, v, w);
}

fn raystart(screenPos: vec2<f32>) -> Ray {
    // Normalized Device Coordinates (NDC)
    let ndc_x = (screenPos.x + 0.5) / screen_width * 2.0 - 1.0;
    let ndc_y = 1.0 - ((screenPos.y + 0.5) / screen_height * 2.0); // Flip Y-axis

    let aspect_ratio = screen_width / screen_height;
    let fov_adjustment = tan(fov / 2.0);

    // Compute the ray direction
    let ray_dir = normalize(
        ndc_x * aspect_ratio * fov_adjustment * camera.side + ndc_y * fov_adjustment * camera.up + camera.direction
    );

    return Ray(camera.position, ray_dir);
}

