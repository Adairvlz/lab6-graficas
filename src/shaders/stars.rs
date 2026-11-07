use super::{mix, noise_helpers::fbm, smoothstep};
use crate::{fragment::Fragment, Uniforms};
use glam::Vec3;

pub fn sun(f: &Fragment, u: &Uniforms) -> Vec3 {
    let p = f.object_position;
    let t = u.time * 0.14;
    // Adveccion del ruido: evoluciona el material sin rotar los vertices.
    let warp = Vec3::new(
        fbm(&u.noise, p * 3.0 + Vec3::new(t, 0.0, 0.0), 3),
        fbm(&u.noise, p * 3.0 + Vec3::new(9.0, t, 0.0), 3),
        0.0,
    );
    let granules = fbm(&u.noise, p * 19.0 + warp * 2.6 + Vec3::splat(t), 3);
    let fissures = (granules * 9.0).sin().abs();
    let spots = 1.0
        - smoothstep(
            -0.38,
            -0.19,
            fbm(&u.noise, p * 5.2 + Vec3::new(0.0, t, 0.0), 3),
        );
    let heat = smoothstep(-0.45, 0.45, granules);
    let base = mix(
        Vec3::new(0.95, 0.12, 0.018),
        Vec3::new(1.0, 0.84, 0.25),
        heat,
    );
    let base = mix(
        base,
        Vec3::new(1.0, 0.96, 0.66),
        smoothstep(0.65, 1.0, fissures) * 0.3,
    );
    // Emision independiente de la direccion de la luz; borde mas frio.
    mix(base, Vec3::new(0.16, 0.025, 0.012), spots * 0.85) * (0.72 + 0.28 * f.normal.z.abs())
}
