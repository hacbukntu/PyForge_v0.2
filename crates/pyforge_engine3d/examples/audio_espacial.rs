//! Reproduce un WAV mono o estéreo como fuente espacial.
//!
//! Uso:
//! cargo run --manifest-path crates/pyforge_engine3d/Cargo.toml \
//!   --example audio_espacial -- sonido.wav auto

use glam::Vec3;
use pyforge_engine3d::audio::{play_spatial_async, AudioBackend, AudioClip, SpatialSource};
use std::path::Path;

fn main() -> Result<(), String> {
    let mut arguments = std::env::args().skip(1);
    let path = arguments
        .next()
        .ok_or("USO: audio_espacial <archivo.wav> [auto|pipewire|alsa]")?;
    let backend = match arguments.next().as_deref().unwrap_or("auto") {
        "auto" => AudioBackend::Auto,
        "pipewire" => AudioBackend::PipeWire,
        "alsa" => AudioBackend::Alsa,
        value => return Err(format!("backend desconocido: {value}")),
    };

    let clip = AudioClip::load_wav(Path::new(&path))?;
    let source = SpatialSource {
        // Una posición positiva en X se escucha con mayor intensidad a la derecha.
        position: Vec3::new(2.0, 0.0, 1.0),
        volume: 0.8,
        max_distance: 20.0,
    };
    let playback = play_spatial_async(clip, source, Vec3::ZERO, Vec3::X, backend);
    playback
        .join()
        .map_err(|_| "el hilo de audio terminó inesperadamente".to_string())?
}
