use glam::Vec3;

#[derive(Clone, Copy, Default)]
pub struct Fragment {
    pub object_position: Vec3,
    pub normal: Vec3,
    pub depth: f32,
    pub covered: bool,
}
