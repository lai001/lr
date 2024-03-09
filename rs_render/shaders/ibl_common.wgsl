// http://holger.dammertz.org/stuff/notes_HammersleyOnHemisphere.html
fn radical_inverse_vd_c(in_bits: u32) -> f32 {
    var bits = in_bits;
    bits = (bits << 16u) | (bits >> 16u);
    bits = ((bits & 0x55555555u) << 1u) | ((bits & 0xAAAAAAAAu) >> 1u);
    bits = ((bits & 0x33333333u) << 2u) | ((bits & 0xCCCCCCCCu) >> 2u);
    bits = ((bits & 0x0F0F0F0Fu) << 4u) | ((bits & 0xF0F0F0F0u) >> 4u);
    bits = ((bits & 0x00FF00FFu) << 8u) | ((bits & 0xFF00FF00u) >> 8u);
    return f32(bits) * 2.3283064365386963e-10; // / 0x100000000
}

fn hammersley_2d(i: u32, n: u32) -> vec2<f32> {
    return vec2<f32>(f32(i) / f32(n), radical_inverse_vd_c(i));
    // return float2((float)i / (float)n, van_der_corpus(i, 2u));
}

fn importance_sample_ggx(Xi: vec2<f32>, roughness: f32) -> vec3<f32> {
    var a = roughness * roughness;
    var phi = 2.0 * PI * Xi.x;
    var cosTheta = sqrt((1.0 - Xi.y) / (1.0 + (a * a - 1.0) * Xi.y));
    var sinTheta = sqrt(1.0 - cosTheta * cosTheta);
    var H: vec3<f32>;
    H.x = cos(phi) * sinTheta;
    H.y = sin(phi) * sinTheta;
    H.z = cosTheta;
    return H;
}

fn geometry_schlick_ggx(n_dot_v: f32, roughness: f32) -> f32 {
    var a = roughness;
    var k = (a * a) / 2.0;
    var nom = n_dot_v;
    var denom = n_dot_v * (1.0 - k) + k;
    return nom / denom;
}

fn geometry_smith(n: vec3<f32>, v: vec3<f32>, l: vec3<f32>, roughness: f32) -> f32 {
    var n_dot_v = max(0.0, dot(n, v));
    var n_dot_l = max(0.0, dot(n, l));
    var ggx2 = geometry_schlick_ggx(n_dot_v, roughness);
    var ggx1 = geometry_schlick_ggx(n_dot_l, roughness);
    return ggx1 * ggx2;
}