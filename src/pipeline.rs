use crate::{fragment::Fragment, mesh::Vertex};
use glam::{Mat3, Vec2, Vec3};

#[derive(Clone, Copy)]
struct Transformed {
    screen: Vec2,
    world: Vec3,
    object: Vec3,
    normal: Vec3,
}

fn vertex_shader(
    v: &Vertex,
    rotation: Mat3,
    center: Vec3,
    radius: f32,
    size: usize,
) -> Transformed {
    // Normalizacion como transformacion de modelo; el OBJ no se altera.
    let p = (v.position - center) / radius;
    let world = rotation * p;
    Transformed {
        screen: Vec2::new(world.x, -world.y) * size as f32 * 0.405 + Vec2::splat(size as f32 * 0.5),
        world,
        object: p,
        // Normal radial de esfera para sombreado suave. No cambia la geometria.
        normal: rotation * p.normalize_or_zero(),
    }
}
fn edge(a: Vec2, b: Vec2, p: Vec2) -> f32 {
    (b - a).perp_dot(p - a)
}

// Proyeccion ortografica: las coordenadas baricentricas interpolan directamente.
pub fn rasterize(vertices: &[Vertex], size: usize, angle: f32) -> Vec<Fragment> {
    let min = vertices
        .iter()
        .fold(Vec3::splat(f32::INFINITY), |a, v| a.min(v.position));
    let max = vertices
        .iter()
        .fold(Vec3::splat(f32::NEG_INFINITY), |a, v| a.max(v.position));
    let center = (min + max) * 0.5;
    let radius = vertices
        .iter()
        .map(|v| (v.position - center).length())
        .fold(0.0_f32, f32::max)
        .max(0.001);
    let rotation = Mat3::from_rotation_y(angle);
    let transformed: Vec<_> = vertices
        .iter()
        .map(|v| vertex_shader(v, rotation, center, radius, size))
        .collect();
    let mut pixels = vec![
        Fragment {
            depth: f32::INFINITY,
            ..Default::default()
        };
        size * size
    ];
    for tri in transformed.as_chunks::<3>().0 {
        let [a, b, c] = [tri[0], tri[1], tri[2]];
        let area = edge(a.screen, b.screen, c.screen);
        if area.abs() < 0.00001 {
            continue;
        }
        let lo = a.screen.min(b.screen).min(c.screen).floor().max(Vec2::ZERO);
        let hi = a
            .screen
            .max(b.screen)
            .max(c.screen)
            .ceil()
            .min(Vec2::splat((size - 1) as f32));
        for y in lo.y as usize..=hi.y as usize {
            for x in lo.x as usize..=hi.x as usize {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let u = edge(b.screen, c.screen, p) / area;
                let v = edge(c.screen, a.screen, p) / area;
                let w = 1.0 - u - v;
                if u < -0.00001 || v < -0.00001 || w < -0.00001 {
                    continue;
                }
                let depth = -(a.world.z * u + b.world.z * v + c.world.z * w);
                let pixel = &mut pixels[y * size + x];
                if depth < pixel.depth {
                    *pixel = Fragment {
                        object_position: a.object * u + b.object * v + c.object * w,
                        normal: (a.normal * u + b.normal * v + c.normal * w).normalize_or_zero(),
                        depth,
                        covered: true,
                    };
                }
            }
        }
    }
    pixels
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sphere_has_finite_fragments_and_front_depth() {
        let mesh = crate::mesh::load(std::path::Path::new("assets/sphere.obj")).unwrap();
        let pixels = rasterize(&mesh, 64, 0.0);
        let center = pixels[32 * 64 + 32];
        assert!(center.covered && center.depth < -0.8);
        assert!(!pixels[0].covered);
        assert!(pixels
            .iter()
            .filter(|p| p.covered)
            .all(|p| p.normal.is_finite() && p.object_position.is_finite()));
    }
}
