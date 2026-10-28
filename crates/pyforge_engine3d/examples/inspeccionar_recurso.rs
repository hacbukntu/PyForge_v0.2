//! Carga de forma acotada un OBJ, glTF, PNG o JPEG e informa su contenido.
//!
//! Este ejemplo inspecciona el recurso en CPU; no afirma que materiales o
//! animaciones glTF estén conectados al pipeline visible.

use pyforge_engine3d::assets::{load_gltf, load_obj, load_texture};
use std::path::Path;

fn main() -> Result<(), String> {
    let argument = std::env::args()
        .nth(1)
        .ok_or("USO: inspeccionar_recurso <modelo.obj|modelo.gltf|textura.png>")?;
    let path = Path::new(&argument);
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    match extension.as_str() {
        "obj" => print_meshes(load_obj(path)?),
        "gltf" | "glb" => print_meshes(load_gltf(path)?),
        "png" | "jpg" | "jpeg" => {
            let texture = load_texture(path)?;
            println!(
                "Textura: {} x {} ({} bytes RGBA)",
                texture.width,
                texture.height,
                texture.rgba.len()
            );
        }
        _ => return Err("extensión admitida: .obj, .gltf, .glb, .png, .jpg o .jpeg".into()),
    }
    Ok(())
}

fn print_meshes(meshes: Vec<pyforge_engine3d::assets::MeshData>) {
    println!("Mallas encontradas: {}", meshes.len());
    for (index, mesh) in meshes.iter().enumerate() {
        println!(
            "  #{index}: {} vértices, {} normales, {} UV, {} índices",
            mesh.positions.len(),
            mesh.normals.len(),
            mesh.texcoords.len(),
            mesh.indices.len()
        );
    }
}
