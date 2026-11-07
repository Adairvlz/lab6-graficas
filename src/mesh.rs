use glam::Vec3;
use std::{error::Error, path::Path};

pub struct Vertex {
    pub position: Vec3,
}

// La misma malla del profesor se carga una sola vez para todos los materiales.
pub fn load(path: &Path) -> Result<Vec<Vertex>, Box<dyn Error>> {
    let (models, _) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )?;
    let mut vertices = Vec::new();
    for model in models {
        let m = model.mesh;
        for i in m.indices {
            let j = i as usize * 3;
            let p = Vec3::from_slice(&m.positions[j..j + 3]);
            vertices.push(Vertex { position: p });
        }
    }
    if vertices.is_empty() {
        return Err("El OBJ no contiene triangulos".into());
    }
    Ok(vertices)
}
