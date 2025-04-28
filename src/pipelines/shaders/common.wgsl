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

fn hash33(p: vec3<f32>) -> vec3<f32> {
    var p3 = fract(p * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 += dot(p3, p3.yxz + 33.33);
    return fract((p3.xxy + p3.yxx) * p3.zyx);
}

fn rand_f(state: ptr<function, u32>) -> f32 {
    *state = *state * 747796405u + 2891336453u;
    let word = ((*state >> ((*state >> 28u) + 4u)) ^ *state) * 277803737u;
    return f32((word >> 22u) ^ word) * bitcast<f32>(0x2f800004u);
}

fn rand_vec2f(state: ptr<function, u32>) -> vec2<f32> {
    return vec2(rand_f(state), rand_f(state));
}

fn rand_vec3f(state: ptr<function, u32>) -> vec3<f32> {
    return vec3(rand_f(state), rand_f(state), rand_f(state));
}

fn get_rand_vector_aligned(p: vec3<f32>, i: u32) -> vec3<f32> {
    let offset = vec3<f32>(i);
    let sample = hash33(p + offset);
    if dot(p, sample) > 0.0 {
        return sample;
    }
    return -sample;
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

fn linear_to_gamma(c: vec4<f32>) -> vec4<f32> {
    return vec4<f32>(
        linear_to_gamma_comp(c.r),
        linear_to_gamma_comp(c.g),
        linear_to_gamma_comp(c.b),
        c.a);
}

fn linear_to_gamma_comp(c: f32) -> f32 {
    if c <= 0.0 {
        return 0.0;
    }
    return sqrt(c);
}

