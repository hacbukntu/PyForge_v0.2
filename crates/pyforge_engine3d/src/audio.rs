//! Audio espacial y salida física desacoplada del renderer.
//!
//! La mezcla se realiza dentro del motor para que sea determinista y comprobable.
//! La última etapa envía PCM S16 estéreo a PipeWire (`pw-cat`) o ALSA (`aplay`).
//! Ambos procesos se ejecutan con argumentos fijos, sin intérprete de comandos.

use glam::Vec3;
use std::{
    io::Write,
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    thread,
};

/// Sonido mono normalizado que el mezclador puede ubicar en el espacio 3D.
#[derive(Clone, Debug)]
pub struct AudioClip {
    pub sample_rate: u32,
    pub samples: Vec<f32>,
}

impl AudioClip {
    /// Carga hasta diez minutos de WAV PCM de uno o dos canales.
    pub fn load_wav(path: &Path) -> Result<Self, String> {
        let mut reader = hound::WavReader::open(path).map_err(|e| format!("WAV: {e}"))?;
        let spec = reader.spec();
        if spec.channels == 0
            || spec.channels > 2
            || spec.sample_rate < 8_000
            || spec.sample_rate > 192_000
        {
            return Err("formato WAV fuera de límites".into());
        }
        let mut raw = Vec::new();
        for sample in reader.samples::<i16>().take(192_000usize * 60 * 10 * 2) {
            raw.push(sample.map_err(|e| format!("WAV: {e}"))? as f32 / 32768.0);
        }
        let samples = if spec.channels == 1 {
            raw
        } else {
            raw.chunks_exact(2).map(|x| (x[0] + x[1]) * 0.5).collect()
        };
        Ok(Self {
            sample_rate: spec.sample_rate,
            samples,
        })
    }

    /// Produce muestras izquierda/derecha según la posición de fuente y oyente.
    pub fn spatial_stereo(
        &self,
        source: &SpatialSource,
        listener: Vec3,
        right: Vec3,
    ) -> Vec<[f32; 2]> {
        let (left_gain, right_gain) = source.gains(listener, right);
        self.samples
            .iter()
            .map(|sample| [sample * left_gain, sample * right_gain])
            .collect()
    }
}

/// Fuente puntual con atenuación lineal y paneo estéreo.
#[derive(Clone, Debug)]
pub struct SpatialSource {
    pub position: Vec3,
    pub volume: f32,
    pub max_distance: f32,
}

impl SpatialSource {
    /// Calcula las ganancias de ambos oídos sin permitir valores fuera de rango.
    pub fn gains(&self, listener: Vec3, right: Vec3) -> (f32, f32) {
        let delta = self.position - listener;
        let distance = delta.length();
        let attenuation = (1.0 - distance / self.max_distance.max(0.001)).clamp(0.0, 1.0)
            * self.volume.clamp(0.0, 1.0);
        let pan = if distance > 0.0 && right.length_squared() > 0.0 {
            delta.normalize().dot(right.normalize()).clamp(-1.0, 1.0)
        } else {
            0.0
        };
        (
            attenuation * (1.0 - pan) * 0.5,
            attenuation * (1.0 + pan) * 0.5,
        )
    }
}

/// Dispositivo solicitado para la reproducción física.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AudioBackend {
    /// Prefiere PipeWire y usa ALSA si `pw-cat` no está instalado.
    Auto,
    PipeWire,
    Alsa,
}

/// Proceso de audio abierto. Al finalizar se cierra su entrada y se espera al hijo.
pub struct AudioPlayback {
    backend: AudioBackend,
    child: Child,
    input: Option<ChildStdin>,
}

impl AudioPlayback {
    /// Backend físico elegido después de resolver `Auto`.
    pub fn backend(&self) -> AudioBackend {
        self.backend
    }

    /// Envía un bloque de PCM estéreo. Los flotantes se saturan antes de convertirlos.
    pub fn write_stereo(&mut self, frames: &[[f32; 2]]) -> Result<(), String> {
        let input = self
            .input
            .as_mut()
            .ok_or_else(|| "la salida de audio ya fue cerrada".to_string())?;
        let mut bytes = Vec::with_capacity(frames.len() * 4);
        for frame in frames {
            for sample in frame {
                let value = (sample.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16;
                bytes.extend_from_slice(&value.to_le_bytes());
            }
        }
        input
            .write_all(&bytes)
            .map_err(|e| format!("no se pudo enviar audio al dispositivo: {e}"))
    }

    /// Cierra el flujo y comprueba que el servidor de audio lo aceptó.
    pub fn finish(mut self) -> Result<(), String> {
        drop(self.input.take());
        let status = self
            .child
            .wait()
            .map_err(|e| format!("no se pudo esperar la salida de audio: {e}"))?;
        if status.success() {
            Ok(())
        } else {
            Err(format!(
                "el backend de audio {:?} terminó con {status}",
                self.backend
            ))
        }
    }
}

impl Drop for AudioPlayback {
    fn drop(&mut self) {
        // Evita dejar procesos huérfanos cuando el juego abandona la reproducción.
        drop(self.input.take());
        let _ = self.child.try_wait();
    }
}

/// Abre una salida PCM S16LE estéreo sin imponer dependencias de desarrollo.
pub fn open_output(backend: AudioBackend, sample_rate: u32) -> Result<AudioPlayback, String> {
    if !(8_000..=192_000).contains(&sample_rate) {
        return Err("frecuencia de audio fuera del intervalo 8000..192000".into());
    }
    let selected = match backend {
        AudioBackend::Auto if command_available("pw-cat") => AudioBackend::PipeWire,
        AudioBackend::Auto if command_available("aplay") => AudioBackend::Alsa,
        AudioBackend::Auto => return Err("no se encontró pw-cat (PipeWire) ni aplay (ALSA)".into()),
        explicit => explicit,
    };
    let rate = sample_rate.to_string();
    let mut command = match selected {
        AudioBackend::PipeWire => {
            let mut command = Command::new("pw-cat");
            command.args([
                "--playback",
                "--raw",
                "--format=s16",
                "--channels=2",
                "--rate",
                &rate,
                "-",
            ]);
            command
        }
        AudioBackend::Alsa => {
            let mut command = Command::new("aplay");
            command.args(["-q", "-t", "raw", "-f", "S16_LE", "-c", "2", "-r", &rate]);
            command
        }
        AudioBackend::Auto => unreachable!("Auto se resuelve antes de crear el proceso"),
    };
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("no se pudo iniciar el backend {selected:?}: {e}"))?;
    let input = child
        .stdin
        .take()
        .ok_or_else(|| "el backend de audio no proporcionó entrada estándar".to_string())?;
    Ok(AudioPlayback {
        backend: selected,
        child,
        input: Some(input),
    })
}

/// Mezcla y reproduce un clip en otro hilo para no bloquear el bucle del juego.
pub fn play_spatial_async(
    clip: AudioClip,
    source: SpatialSource,
    listener: Vec3,
    right: Vec3,
    backend: AudioBackend,
) -> thread::JoinHandle<Result<(), String>> {
    thread::spawn(move || {
        let frames = clip.spatial_stereo(&source, listener, right);
        let mut output = open_output(backend, clip.sample_rate)?;
        output.write_stereo(&frames)?;
        output.finish()
    })
}

/// Comprueba la presencia de una orden sin ejecutar una shell.
fn command_available(name: &str) -> bool {
    Command::new(name)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frecuencia_invalida_no_inicia_un_proceso() {
        assert!(open_output(AudioBackend::Auto, 1).is_err());
        assert!(open_output(AudioBackend::Auto, 500_000).is_err());
    }

    #[test]
    fn mezcla_satura_volumen_y_atenua_distancia() {
        let source = SpatialSource {
            position: Vec3::X,
            volume: 2.0,
            max_distance: 10.0,
        };
        let (left, right) = source.gains(Vec3::ZERO, Vec3::X);
        assert!(right > left);
        assert!((0.0..=1.0).contains(&right));

        let silent = SpatialSource {
            position: Vec3::X * 20.0,
            volume: 1.0,
            max_distance: 10.0,
        };
        assert_eq!(silent.gains(Vec3::ZERO, Vec3::X), (0.0, 0.0));
    }
}
