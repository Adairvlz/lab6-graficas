use glam::Vec3;
use noise::{NoiseFn, OpenSimplex};
pub fn sample(n: &OpenSimplex, p: Vec3) -> f32 {
    n.get([p.x as f64, p.y as f64, p.z as f64]) as f32
}
pub fn fbm(n: &OpenSimplex, mut p: Vec3, octaves: u32) -> f32 {
    let (mut value, mut amplitude, mut total) = (0.0, 0.5, 0.0);
    for _ in 0..octaves {
        value += sample(n, p) * amplitude;
        total += amplitude;
        p = p * 2.03 + Vec3::splat(7.1);
        amplitude *= 0.5;
    }
    value / total
}
