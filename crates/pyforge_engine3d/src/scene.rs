//! Formato versionado y acotado que conecta ST, importadores, editor y renderer.
use glam::{Mat4, Quat, Vec3};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

pub const SCENE_VERSION: u32 = 1;
pub const MAX_ENTITIES: usize = 100_000;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transform {
    pub position: [f32; 3],
    pub rotation: [f32; 4],
    pub scale: [f32; 3],
}
impl Default for Transform {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale: [1.0; 3],
        }
    }
}
impl Transform {
    pub fn matrix(&self) -> Mat4 {
        Mat4::from_scale_rotation_translation(
            Vec3::from_array(self.scale),
            Quat::from_array(self.rotation).normalize(),
            Vec3::from_array(self.position),
        )
    }
    fn valid(&self) -> bool {
        self.position
            .iter()
            .chain(&self.rotation)
            .chain(&self.scale)
            .all(|x| x.is_finite())
            && self.scale.iter().all(|x| *x != 0.0)
            && Quat::from_array(self.rotation).length_squared() > 1e-12
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entity {
    pub name: String,
    pub transform: Transform,
    pub mesh: Option<String>,
    pub material: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Scene {
    pub version: u32,
    pub name: String,
    pub entities: Vec<Entity>,
}
impl Scene {
    pub fn validate(&self) -> Result<(), String> {
        if self.version != SCENE_VERSION {
            return Err(format!("versión de escena no compatible: {}", self.version));
        }
        if self.name.is_empty() || self.name.len() > 256 {
            return Err("nombre de escena inválido".into());
        }
        if self.entities.len() > MAX_ENTITIES {
            return Err("demasiadas entidades".into());
        }
        for entity in &self.entities {
            if entity.name.is_empty() || entity.name.len() > 256 || !entity.transform.valid() {
                return Err(format!("entidad inválida: {}", entity.name));
            }
        }
        Ok(())
    }
    pub fn from_json(text: &str) -> Result<Self, String> {
        if text.len() > 16 * 1024 * 1024 {
            return Err("escena mayor de 16 MiB".into());
        }
        let scene: Self = serde_json::from_str(text).map_err(|e| format!("JSON de escena: {e}"))?;
        scene.validate()?;
        Ok(scene)
    }
    pub fn load(path: &Path) -> Result<Self, String> {
        let metadata = path.metadata().map_err(|e| format!("abrir escena: {e}"))?;
        if metadata.len() > 16 * 1024 * 1024 {
            return Err("escena mayor de 16 MiB".into());
        }
        let text = fs::read_to_string(path).map_err(|e| format!("leer escena: {e}"))?;
        Self::from_json(&text)
    }
    pub fn save_atomic(&self, path: &Path) -> Result<(), String> {
        self.validate()?;
        if path.file_name().is_none() {
            return Err("ruta de escena inválida".into());
        }
        let parent = path.parent().unwrap_or(Path::new("."));
        let temporary = parent.join(format!(
            ".{}.tmp",
            path.file_name().unwrap().to_string_lossy()
        ));
        let data =
            serde_json::to_vec_pretty(self).map_err(|e| format!("serializar escena: {e}"))?;
        fs::write(&temporary, data).map_err(|e| format!("guardar escena temporal: {e}"))?;
        if let Err(error) = fs::rename(&temporary, path) {
            let _ = fs::remove_file(&temporary);
            return Err(format!("publicar escena: {error}"));
        }
        Ok(())
    }
}
