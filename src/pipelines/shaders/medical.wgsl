struct Ray {
    start: vec3<f32>,
    direction: vec3<f32>,
}

struct RayResult {
    color: vec4<f32>
    // hit: Hit
}

struct Details {
    screen_width: f32,
    screen_height: f32
}

@group(0) @binding(0)
var color_buffer: texture_storage_2d<rgba8unorm, write>;

@group(0) @binding(1)
var input: texture_3d<f32>;

@group(0) @binding(2)
var<uniform> camera: CameraUniform;

@group(0) @binding(3)
var<uniform> details: Details;

@group(0) @binding(4)
var input_sampler: sampler;

var<workgroup> counter: atomic<u32>;

const numSamples: u32 = 10;
const useAA: bool = true;
const numBounceSamples: u32 = 10;
const numBounces: u32 = 3;
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

    var color: vec4<f32>;
    if useAA {
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
    } else {
        color = castray(vec2<f32>(screen_pos_u32), &rng);
    }

    color = linear_to_gamma(color);
    let screen_pos_i32 = vec2<i32>(screen_pos_u32);
    textureStore(color_buffer, screen_pos_i32, color);
}

fn castray(screen_pos: vec2<f32>, rng: ptr<function, u32>) -> vec4<f32> {
    var ray = raystart(screen_pos, rng);
    let hitResult = scene_hit(ray);
    return hitResult.color;
}

fn scene_hit(ray: Ray) -> RayResult {
    var max = -10000.0;
    for(var i = 0; i < 100; i++)
    {
        let pos = ray.start + ray.direction * f32(i);
        let sample:vec4<f32> = textureSampleLevel(input, input_sampler, pos, 0.0);
        if(sample.r > max)
        {
            max = sample.r;
          }
    }
    let background = vec4<f32>(max/1000.0, max/1000.0, max/1000.0, 1.0);
    return RayResult(background);
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
