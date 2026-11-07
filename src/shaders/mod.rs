pub mod noise_helpers;
pub mod planets;
pub mod stars;
use crate::{fragment::Fragment, Uniforms};
use glam::Vec3;
pub fn mix(a: Vec3, b: Vec3, t: f32) -> Vec3 {
    a.lerp(b, t.clamp(0.0, 1.0))
}
pub fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
pub fn fragment_shader(f: &Fragment, u: &Uniforms) -> Vec3 {
    let color = match u.body {
        0 => stars::sun(f, u),
        1 => planets::rocky(f, u),
        _ => planets::gas(f, u),
    };
    if u.grayscale {
        Vec3::splat(color.dot(Vec3::new(0.2126, 0.7152, 0.0722)))
    } else {
        color
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use noise::OpenSimplex;
    #[test]
    fn materials_are_finite_and_sun_ignores_external_light() {
        let mut u = Uniforms {
            time: 0.0,
            angle: 0.0,
            light: Vec3::Z,
            body: 0,
            grayscale: false,
            noise: OpenSimplex::new(42),
        };
        let f = Fragment {
            object_position: Vec3::Z,
            normal: Vec3::Z,
            depth: -1.0,
            covered: true,
        };
        let sun = fragment_shader(&f, &u);
        u.light = -Vec3::Z;
        assert_eq!(sun, fragment_shader(&f, &u));
        u.time = 3.0;
        assert!((sun - fragment_shader(&f, &u)).length() > 0.001);
        for body in 0..3 {
            u.body = body;
            for p in [Vec3::X, -Vec3::X, Vec3::Y, -Vec3::Y, Vec3::Z, -Vec3::Z] {
                let f = Fragment {
                    object_position: p,
                    normal: p,
                    ..f
                };
                assert!(fragment_shader(&f, &u).is_finite());
            }
        }
    }
}
