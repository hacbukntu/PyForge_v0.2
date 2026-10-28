//! Importadores con límites; nunca ejecutan contenido del recurso.
use std::path::Path;

pub const MAX_VERTICES: usize = 10_000_000;
#[derive(Clone, Debug, Default)]
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub texcoords: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}
#[derive(Clone, Debug)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
}

pub fn load_texture(path: &Path) -> Result<TextureData, String> {
    let metadata = path.metadata().map_err(|e| format!("textura: {e}"))?;
    if metadata.len() > 128 * 1024 * 1024 {
        return Err("textura mayor de 128 MiB".into());
    }
    let image = image::ImageReader::open(path)
        .map_err(|e| format!("textura: {e}"))?
        .decode()
        .map_err(|e| format!("textura: {e}"))?;
    if image.width() == 0 || image.height() == 0 || image.width() > 8192 || image.height() > 8192 {
        return Err("dimensiones de textura inválidas".into());
    }
    let rgba = image.to_rgba8();
    Ok(TextureData {
        width: rgba.width(),
        height: rgba.height(),
        rgba: rgba.into_raw(),
    })
}

pub fn load_wgsl(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("shader WGSL: {e}"))?;
    if bytes.is_empty() || bytes.len() > 1024 * 1024 {
        return Err("shader WGSL vacío o mayor de 1 MiB".into());
    }
    String::from_utf8(bytes).map_err(|_| "shader WGSL no es UTF-8".into())
}

pub fn load_obj(path: &Path) -> Result<Vec<MeshData>, String> {
    let metadata = path.metadata().map_err(|e| format!("OBJ: {e}"))?;
    if metadata.len() > 256 * 1024 * 1024 {
        return Err("OBJ mayor de 256 MiB".into());
    }
    let options = tobj::LoadOptions {
        triangulate: true,
        single_index: true,
        ..Default::default()
    };
    let (models, _) = tobj::load_obj(path, &options).map_err(|e| format!("OBJ: {e}"))?;
    let mut out = Vec::with_capacity(models.len());
    for model in models {
        let m = model.mesh;
        if m.positions.len() % 3 != 0 || m.positions.len() / 3 > MAX_VERTICES {
            return Err("OBJ excede límites".into());
        }
        out.push(MeshData {
            positions: m
                .positions
                .chunks_exact(3)
                .map(|v| [v[0], v[1], v[2]])
                .collect(),
            normals: m
                .normals
                .chunks_exact(3)
                .map(|v| [v[0], v[1], v[2]])
                .collect(),
            texcoords: m.texcoords.chunks_exact(2).map(|v| [v[0], v[1]]).collect(),
            indices: m.indices,
        });
    }
    Ok(out)
}

pub fn load_gltf(path: &Path) -> Result<Vec<MeshData>, String> {
    let metadata = path.metadata().map_err(|e| format!("glTF: {e}"))?;
    if metadata.len() > 256 * 1024 * 1024 {
        return Err("glTF mayor de 256 MiB".into());
    }
    let (document, buffers, _) = gltf::import(path).map_err(|e| format!("glTF: {e}"))?;
    let mut out = Vec::new();
    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|b| Some(&buffers[b.index()]));
            let positions: Vec<_> = reader
                .read_positions()
                .ok_or("glTF sin posiciones")?
                .collect();
            if positions.len() > MAX_VERTICES {
                return Err("glTF excede límites".into());
            }
            out.push(MeshData {
                normals: reader
                    .read_normals()
                    .map(|x| x.collect())
                    .unwrap_or_default(),
                texcoords: reader
                    .read_tex_coords(0)
                    .map(|x| x.into_f32().collect())
                    .unwrap_or_default(),
                indices: reader
                    .read_indices()
                    .map(|x| x.into_u32().collect())
                    .unwrap_or_else(|| (0..positions.len() as u32).collect()),
                positions,
            });
        }
    }
    Ok(out)
}
