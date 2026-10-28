//! Pose esquelética CPU; el renderer puede subir sus matrices a un buffer uniforme.
use glam::{Mat4, Quat, Vec3};

#[derive(Clone, Debug)]
pub struct JointPose {
    pub translation: Vec3,
    pub rotation: Quat,
    pub scale: Vec3,
}
impl JointPose {
    pub fn interpolate(&self, other: &Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            translation: self.translation.lerp(other.translation, t),
            rotation: self.rotation.slerp(other.rotation, t).normalize(),
            scale: self.scale.lerp(other.scale, t),
        }
    }
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
    }
}

pub fn skin_matrices(
    local: &[JointPose],
    parents: &[Option<usize>],
    inverse_bind: &[Mat4],
) -> Result<Vec<Mat4>, String> {
    if local.len() != parents.len() || local.len() != inverse_bind.len() || local.len() > 1024 {
        return Err("esqueleto inválido".into());
    }
    let mut world = vec![Mat4::IDENTITY; local.len()];
    for i in 0..local.len() {
        let local_matrix = local[i].matrix();
        world[i] = match parents[i] {
            Some(p) if p < i => world[p] * local_matrix,
            Some(_) => return Err("jerarquía esquelética no topológica".into()),
            None => local_matrix,
        };
    }
    Ok(world
        .into_iter()
        .zip(inverse_bind)
        .map(|(w, b)| w * *b)
        .collect())
}
