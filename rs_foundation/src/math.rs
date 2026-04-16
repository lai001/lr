pub fn smooth_damp(x: f32, target: f32, k: f32, dt: f32) -> f32 {
    return x + (target - x) * (1.0 - (-k * dt).exp());
}
