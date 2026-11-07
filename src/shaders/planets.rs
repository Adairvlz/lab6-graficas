use super::{mix, noise_helpers::fbm, smoothstep};
use crate::{fragment::Fragment, Uniforms};
use glam::{Mat3, Vec3};

fn clouds(p: Vec3, u: &Uniforms) -> f32 {
    let q = Mat3::from_rotation_y(u.time * 0.065) * p.normalize_or_zero();
    smoothstep(0.02, 0.28, fbm(&u.noise, q * 6.8 + Vec3::splat(23.0), 4))
}

fn night_mask(normal: Vec3, light: Vec3) -> f32 {
    1.0 - smoothstep(-0.22, 0.02, normal.dot(light))
}

fn night_lights(p: Vec3, u: &Uniforms) -> f32 {
    // Agrupaciones y puntos finos anclados al terreno, no al tiempo.
    let settlements = smoothstep(-0.10, 0.22, fbm(&u.noise, p * 12.0, 3));
    let points = smoothstep(0.23, 0.48, fbm(&u.noise, p * 145.0, 2));
    settlements * points
}

fn cloud_shadow(p: Vec3, light: Vec3, u: &Uniforms) -> f32 {
    // Interseccion analitica del rayo hacia la luz con una capa de radio 1+h.
    // No se agrega geometria: se consulta la misma mascara procedural.
    let surface = p.normalize_or_zero();
    let height = 0.075;
    let dot = surface.dot(light);
    let distance = -dot + (dot * dot + (1.0_f32 + height).powi(2) - 1.0).sqrt();
    clouds(surface + light * distance, u)
}
pub fn rocky(f: &Fragment, u: &Uniforms) -> Vec3 {
    let p = f.object_position;
    let terrain = fbm(&u.noise, p * 2.8, 5);
    let land = smoothstep(-0.035, 0.025, terrain);
    let mountains = smoothstep(0.13, 0.38, terrain);
    let dry = smoothstep(
        0.15,
        0.65,
        fbm(&u.noise, p * 5.0 + Vec3::splat(8.0), 3) + 0.5 - p.y.abs() * 0.6,
    );
    let ocean = mix(
        Vec3::new(0.012, 0.055, 0.18),
        Vec3::new(0.02, 0.48, 0.52),
        smoothstep(-0.22, 0.02, terrain),
    );
    let ground = mix(
        Vec3::new(0.10, 0.30, 0.16),
        Vec3::new(0.66, 0.46, 0.24),
        dry,
    );
    let ground = mix(ground, Vec3::new(0.48, 0.43, 0.38), mountains);
    let mut base = mix(ocean, ground, land);
    let ice = smoothstep(0.77, 0.9, p.y.abs() + terrain * 0.16);
    base = mix(base, Vec3::new(0.82, 0.93, 0.96), ice);
    let diffuse = f.normal.dot(u.light).max(0.0);
    // Muestrear la capa alta hacia la luz desplaza la sombra respecto a la nube.
    let local_light = Mat3::from_rotation_y(-u.angle) * u.light;
    let shadow = cloud_shadow(p, local_light, u);
    base *= (0.09 + diffuse * 0.91) * (1.0 - shadow * 0.48 * diffuse);
    let half = (u.light + Vec3::Z).normalize();
    let spec = f.normal.dot(half).max(0.0).powf(75.0) * (1.0 - land) * (1.0 - ice);
    base += Vec3::new(0.7, 0.87, 1.0) * spec * 0.85;
    // Emision nocturna solo sobre tierra sin hielo. Las nubes la ocultan abajo.
    base += Vec3::new(1.0, 0.62, 0.18)
        * night_lights(p, u)
        * night_mask(f.normal, u.light)
        * land
        * (1.0 - ice)
        * 2.4;
    base = mix(
        base,
        Vec3::splat(0.055 + diffuse * 0.945),
        clouds(p, u) * 0.92,
    );
    let rim = (1.0 - f.normal.z.abs()).powf(3.5) * smoothstep(-0.2, 0.6, f.normal.dot(u.light));
    base + Vec3::new(0.12, 0.48, 0.92) * rim * 0.65
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn night_emission_switches_with_light_not_time() {
        assert_eq!(night_mask(Vec3::Z, Vec3::Z), 0.0);
        assert_eq!(night_mask(Vec3::Z, -Vec3::Z), 1.0);
        let twilight = night_mask(Vec3::Z, Vec3::X);
        assert!(twilight > 0.0 && twilight < 1.0);
    }
}

pub fn gas(f: &Fragment, u: &Uniforms) -> Vec3 {
    let p = f.object_position;
    let lon = p.x.atan2(p.z);
    let lat = p.y.asin();
    // Tormenta eliptica: gira las coordenadas de las bandas dentro del vortice.
    let dx = (lon - 0.35 + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU)
        - std::f32::consts::PI;
    let dy = lat + 0.23;
    let r = ((dx / 0.43).powi(2) + (dy / 0.22).powi(2)).sqrt();
    let mask = 1.0 - smoothstep(0.75, 1.35, r);
    let theta = dy.atan2(dx) + 4.0 * (1.0 - r).max(0.0) + u.time * 0.22;
    let turbulence = fbm(&u.noise, p * 6.0 + Vec3::new(u.time * 0.035, 0.0, 0.0), 4);
    let warped = p.y + 0.08 * turbulence + mask * theta.sin() * 0.11;
    let wave = (warped * 37.0 + turbulence * 3.4).sin() * 0.5 + 0.5;
    let base = mix(
        Vec3::new(0.14, 0.27, 0.39),
        Vec3::new(0.76, 0.85, 0.80),
        smoothstep(0.2, 0.8, wave),
    );
    let base = mix(
        base,
        Vec3::new(0.83, 0.53, 0.30),
        smoothstep(0.73, 0.98, (warped * 18.0 + 1.2).sin() * 0.5 + 0.5) * 0.85,
    );
    let rings = (r * 24.0 - theta * 2.0 + turbulence * 3.0).sin() * 0.5 + 0.5;
    let storm = mix(
        Vec3::new(0.26, 0.11, 0.09),
        Vec3::new(0.99, 0.71, 0.39),
        rings,
    );
    let color = mix(base, storm, mask * 0.88);
    color * (0.13 + 0.87 * f.normal.dot(u.light).max(0.0))
}
