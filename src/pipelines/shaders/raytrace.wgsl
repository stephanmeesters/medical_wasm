struct CameraUniform {
    eye: vec3<f32>,
    lens_radius: f32,
    u_axis: vec3<f32>, // camera "right"
    z_near: f32,
    v_axis: vec3<f32>, // camera "up"
    z_far: f32,
    w_axis: vec3<f32>, // camera looks "backwards"
    projection: u32,   // ortho: 0, projection: 1
    horizontal: vec3<f32>,
    _pad2: f32,
    vertical: vec3<f32>,
    _pad3: f32,
    lower_left_corner: vec3<f32>,
    _pad4: f32
};

struct Ray {
    start: vec3<f32>,
    direction: vec3<f32>,
}

struct Hit {
    hit: bool,
    t: f32,
    p: vec3<f32>,
    normal: vec3<f32>,
    center: vec3<f32>
}

struct RayResult {
    color: vec4<f32>,
    hit: Hit
}

struct Sphere {
    pos: vec3<f32>,
    radius: f32,
    material: vec4<f32> 
}

struct Details {
    screen_width: f32,
    screen_height: f32
}

@group(0) @binding(0)
var color_buffer: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var<uniform> camera: CameraUniform;

@group(0) @binding(2)
var<storage> spheres: array<Sphere>;

@group(0) @binding(3)
var<uniform> details: Details;

var<workgroup> counter: atomic<u32>;

const numSamples: u32 = 10;
const useAA: bool = true;
const numBounceSamples: u32 = 10;
const scattering: f32 = 0.0;

////

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) GlobalInvocationID: vec3<u32>) {
    let screen_size: vec2<u32> = textureDimensions(color_buffer);
    let screen_pos_u32: vec2<u32> = GlobalInvocationID.xy;
    let global_id_1d: u32 = screen_pos_u32.y * screen_size.x + screen_pos_u32.x;

    if screen_pos_u32.x >= screen_size.x || screen_pos_u32.y >= screen_size.y {
        return;
    }

    counter = counter + 1;
    var rng = screen_pos_u32.x + screen_pos_u32.y * 1000u + counter * 5782582u;

    if useAA {
        var color: vec4<f32>;
        let offset = 1.0;
        for (var i: u32 = 0; i < numSamples; i = i + 1) {
            let coord = rand_vec2f(&rng) - vec2<f32>(0.5);
            color += castray(
                vec2<f32>(f32(screen_pos_u32.x) + coord.x * offset,
                    f32(screen_pos_u32.y) + coord.y * offset),
                &rng
            );
        }
        color /= f32(numSamples);
        let screen_pos_i32 = vec2<i32>(screen_pos_u32);
        textureStore(color_buffer, screen_pos_i32, color);
    } else {
        let offset = 0.5;
        var color = castray(vec2<f32>(screen_pos_u32), &rng);
        let screen_pos_i32 = vec2<i32>(screen_pos_u32);
        textureStore(color_buffer, screen_pos_i32, color);
    }
}

fn castray(screen_pos: vec2<f32>, rng: ptr<function, u32>) -> vec4<f32> {
    var ray = raystart(screen_pos, rng);

    let result1 = castray_impl(ray);
    if result1.hit.hit && dot(result1.hit.normal, ray.direction) < 0.0 { // second ray

        var color: vec4<f32> = result1.color;
        var count = 1;
        for (var i: u32 = 0; i < numBounceSamples; i = i + 1) {
            let rand = normalize(rand_vec3f(rng) - vec3<f32>(0.5));
            let zz = result1.hit.p + result1.hit.normal + scattering * rand;
            let newray = Ray(result1.hit.p, -normalize(result1.hit.p - zz));
            let newresult = castray_impl(newray);
            if newresult.hit.hit {
                color += newresult.color;
                count = count + 1;

                let rand2 = normalize(rand_vec3f(rng) - vec3<f32>(0.5));
                let zz2 = newresult.hit.p + newresult.hit.normal + scattering * rand2;
                let newray2 = Ray(newresult.hit.p, -normalize(newresult.hit.p - zz2));
                let newresult2 = castray_impl(newray2);
                if newresult2.hit.hit {
                    color += newresult2.color;
                    count = count + 1;
                }
            }
        }
        color /= f32(count);

        return color;
    }

    return result1.color;
}

fn castray_impl(ray: Ray) -> RayResult {

    var hit_anything = false;
    var closest = Hit(false, 9999999.0, vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0.0));
    var material = vec4<f32>(137.0 / 255.0, 207.0 / 255.0, 240.0 / 255.0, 1.0);

    for (var i = 0u; i < arrayLength(&spheres); i++) {
        let hit_sphere = hit_sphere(spheres[i].pos, spheres[i].radius, ray, 0.01, 1000.0);
        if hit_sphere.hit {
            hit_anything = true;
            if hit_sphere.t < closest.t {
                closest = hit_sphere;
                material = spheres[i].material;
            }
        }
    }

    if hit_anything {
        let color = material;
        return RayResult(color, closest);
    }

    return RayResult(material, closest);
}

fn raystart(screenPos: vec2<f32>, rng: ptr<function, u32>) -> Ray {

    let s = screenPos.x / details.screen_width;
    let t = screenPos.y / details.screen_height;

    if camera.projection == 0 {
        let origin = camera.lower_left_corner + s * camera.horizontal + t * camera.vertical;
        let direction = normalize(-camera.w_axis);
        return Ray(origin, direction);
    } else {
        let ray_target = camera.lower_left_corner + s * camera.horizontal + t * camera.vertical;
        let random_disk = vec2<f32>(camera.lens_radius) * normalize(rand_vec2f(rng));
        let lens_offset = camera.u_axis * random_disk.x + camera.v_axis * random_disk.y;
        let origin = camera.eye + lens_offset;
        let direction = normalize(ray_target - camera.eye - lens_offset);
        return Ray(origin, direction);
    }
}

//// intersections

fn hit_sphere(center: vec3<f32>, radius: f32, r: Ray, near: f32, far: f32) -> Hit {
    let oc = center - r.start;
    let a = dot(r.direction, r.direction);
    let h = dot(oc, r.direction);
    let c = dot(oc, oc) - radius * radius;
    let discriminant = h * h - a * c;

    if discriminant < 0.0 {
        return Hit(false, 0.0, vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0));
    }

    let sqrtd = sqrt(discriminant);
    var root = (h - sqrtd) / a;

    if root <= near || root >= far {
        root = (h + sqrtd) / a;
        if root <= near || root >= far {
            return Hit(false, 0.0, vec3<f32>(0.0), vec3<f32>(0.0), vec3<f32>(0));
        }
    }

    let p = r.start + r.direction * root;
    let normal = normalize(p - center);

    return Hit(true, root, p, normal, center);
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

