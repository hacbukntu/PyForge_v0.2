//! Puente seguro entre un programa ST y el editor visual 3D.
//!
//! El editor no interpreta texto ST ni permite que un script modifique memoria de
//! Rust. `pyforgec` compila el archivo y el ejecutable resultante escribe un
//! protocolo pequeño por su salida estándar. Este módulo limita tiempo, tamaño y
//! cantidad de entidades antes de convertir esa salida en una [`Scene`].

use crate::scene::{Entity, Scene, Transform, SCENE_VERSION};
use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

/// Primera línea que identifica una escena producida para el editor.
pub const PROTOCOL_HEADER: &str = "PYFORGE_EDITOR3D_V1";
/// Última línea del protocolo; detecta salidas incompletas.
pub const PROTOCOL_FOOTER: &str = "FIN_PYFORGE_EDITOR3D_V1";
/// Un script de editor usa un límite menor que el formato general para mantener
/// la interfaz interactiva incluso si el programa ST contiene un error lógico.
pub const MAX_SCRIPT_ENTITIES: usize = 10_000;
const MAX_SCRIPT_SOURCE_BYTES: u64 = 4 * 1024 * 1024;
const MAX_STDOUT_BYTES: usize = 4 * 1024 * 1024;
const MAX_STDERR_BYTES: usize = 512 * 1024;

/// Compila y ejecuta un archivo `.st`, y valida la escena que publica.
///
/// Se invoca `pyforgec` directamente, sin shell. La compilación dispone de 90
/// segundos y el programa ST de 15; ambos procesos se terminan al exceder el
/// límite. El binario temporal se borra tanto en éxito como en error.
pub fn compile_and_run_scene_script(compiler: &Path, source: &Path) -> Result<Scene, String> {
    let source = source
        .canonicalize()
        .map_err(|error| format!("abrir script ST {}: {error}", source.display()))?;
    let metadata = source
        .metadata()
        .map_err(|error| format!("leer metadatos del script ST: {error}"))?;
    if metadata.len() > MAX_SCRIPT_SOURCE_BYTES {
        return Err("el script ST supera 4 MiB".into());
    }
    if source.extension().and_then(|value| value.to_str()) != Some("st") {
        return Err("el archivo de escena debe terminar en .st".into());
    }

    let compiler = resolve_compiler_path(compiler)?;
    let binary = temporary_binary_path();
    let mut build = Command::new(&compiler);
    build
        .arg("build")
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .arg("--no-opt");
    let build_output = run_limited(build, Duration::from_secs(90))?;
    if !build_output.status.success() {
        let _ = fs::remove_file(&binary);
        return Err(format_process_error("compilar la escena ST", &build_output));
    }

    let mut run = Command::new(&binary);
    if let Some(parent) = source.parent() {
        run.current_dir(parent);
    }
    let result = run_limited(run, Duration::from_secs(15));
    let _ = fs::remove_file(&binary);
    let output = result?;
    if !output.status.success() {
        return Err(format_process_error("ejecutar la escena ST", &output));
    }
    let stdout = String::from_utf8(output.stdout)
        .map_err(|_| "la escena ST escribió bytes que no son UTF-8".to_string())?;
    parse_scene_protocol(&stdout)
}

/// Interpreta la salida estricta generada por `editor3d.st`.
///
/// Cada entidad ocupa trece líneas: nombre, malla, material, tres componentes de
/// posición, cuatro de rotación y tres de escala. Una cadena vacía representa un
/// recurso opcional ausente.
pub fn parse_scene_protocol(text: &str) -> Result<Scene, String> {
    if text.len() > MAX_STDOUT_BYTES {
        return Err("la salida ST supera 4 MiB".into());
    }
    let normalized = text.replace("\r\n", "\n");
    let mut cursor = ProtocolCursor::new(&normalized);
    if cursor.next("cabecera")? != PROTOCOL_HEADER {
        return Err(format!(
            "protocolo ST inválido: se esperaba {PROTOCOL_HEADER}; usa importar \"editor3d\""
        ));
    }
    let name = protocol_text(cursor.next("nombre de escena")?, "nombre de escena", false)?;
    let count = cursor
        .next("cantidad de entidades")?
        .parse::<usize>()
        .map_err(|_| "la cantidad de entidades ST no es un entero válido".to_string())?;
    if count > MAX_SCRIPT_ENTITIES {
        return Err(format!(
            "la escena ST contiene {count} entidades; el máximo interactivo es {MAX_SCRIPT_ENTITIES}"
        ));
    }

    let mut entities = Vec::with_capacity(count);
    for index in 0..count {
        let number = index + 1;
        let entity_name = protocol_text(
            cursor.next(&format!("nombre de entidad {number}"))?,
            "nombre de entidad",
            false,
        )?;
        let mesh =
            optional_protocol_text(cursor.next(&format!("malla de entidad {number}"))?, "malla")?;
        let material = optional_protocol_text(
            cursor.next(&format!("material de entidad {number}"))?,
            "material",
        )?;
        let mut values = [0.0_f32; 10];
        for (component, value) in values.iter_mut().enumerate() {
            *value = cursor
                .next(&format!(
                    "transformación {} de entidad {number}",
                    component + 1
                ))?
                .parse::<f32>()
                .map_err(|_| {
                    format!(
                        "la transformación {} de la entidad {number} no es un número",
                        component + 1
                    )
                })?;
            if !value.is_finite() {
                return Err(format!(
                    "la transformación {} de la entidad {number} no es finita",
                    component + 1
                ));
            }
        }
        entities.push(Entity {
            name: entity_name,
            mesh,
            material,
            transform: Transform {
                position: [values[0], values[1], values[2]],
                rotation: [values[3], values[4], values[5], values[6]],
                scale: [values[7], values[8], values[9]],
            },
        });
    }
    if cursor.next("fin del protocolo")? != PROTOCOL_FOOTER {
        return Err(format!("la escena ST debe terminar con {PROTOCOL_FOOTER}"));
    }
    if cursor.any_non_empty() {
        return Err("la escena ST escribió datos después del fin del protocolo".into());
    }

    let scene = Scene {
        version: SCENE_VERSION,
        name,
        entities,
    };
    scene.validate()?;
    Ok(scene)
}

/// Genera un programa ST editable a partir de la escena visual actual.
pub fn scene_to_st(scene: &Scene) -> Result<String, String> {
    scene.validate()?;
    let mut source = String::from(
        "// Escena creada por PyForge Editor 3D.\n// Puedes modificarla y pulsar «Aplicar ST» para reconstruir el viewport.\nimportar \"editor3d\";\n\n",
    );
    source.push_str(&format!(
        "editor3d.iniciar(\"{}\"; {});\n",
        escape_st_protocol_text(&scene.name)?,
        scene.entities.len()
    ));
    for entity in &scene.entities {
        let mesh = entity.mesh.as_deref().unwrap_or("");
        let material = entity.material.as_deref().unwrap_or("");
        source.push_str(&format!(
            "\neditor3d.entidad(\"{}\"; \"{}\"; \"{}\");\n",
            escape_st_protocol_text(&entity.name)?,
            escape_st_protocol_text(mesh)?,
            escape_st_protocol_text(material)?,
        ));
        source.push_str(&format!(
            "editor3d.posicion({}; {}; {});\n",
            st_float(entity.transform.position[0]),
            st_float(entity.transform.position[1]),
            st_float(entity.transform.position[2]),
        ));
        source.push_str(&format!(
            "editor3d.rotacion({}; {}; {}; {});\n",
            st_float(entity.transform.rotation[0]),
            st_float(entity.transform.rotation[1]),
            st_float(entity.transform.rotation[2]),
            st_float(entity.transform.rotation[3]),
        ));
        source.push_str(&format!(
            "editor3d.escala({}; {}; {});\n",
            st_float(entity.transform.scale[0]),
            st_float(entity.transform.scale[1]),
            st_float(entity.transform.scale[2]),
        ));
    }
    source.push_str("editor3d.terminar();\n");
    Ok(source)
}

/// Guarda texto por reemplazo atómico para no dejar un `.st` truncado si falla
/// la escritura.
pub fn save_source_atomic(path: &Path, source: &str) -> Result<(), String> {
    if path.extension().and_then(|value| value.to_str()) != Some("st") {
        return Err("la ruta del script debe terminar en .st".into());
    }
    if source.len() as u64 > MAX_SCRIPT_SOURCE_BYTES {
        return Err("el script ST supera 4 MiB".into());
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    fs::create_dir_all(parent).map_err(|error| format!("crear carpeta del script ST: {error}"))?;
    let name = path
        .file_name()
        .ok_or("ruta del script ST inválida")?
        .to_string_lossy();
    let temporary = parent.join(format!(".{name}.tmp"));
    fs::write(&temporary, source).map_err(|error| format!("guardar script temporal: {error}"))?;
    if let Err(error) = fs::rename(&temporary, path) {
        let _ = fs::remove_file(&temporary);
        return Err(format!("publicar script ST: {error}"));
    }
    Ok(())
}

struct ProtocolCursor<'a> {
    lines: std::str::Lines<'a>,
    number: usize,
}

impl<'a> ProtocolCursor<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            lines: text.lines(),
            number: 0,
        }
    }

    fn next(&mut self, expected: &str) -> Result<&'a str, String> {
        self.number += 1;
        self.lines.next().ok_or_else(|| {
            format!(
                "salida ST incompleta en la línea {}: falta {expected}",
                self.number
            )
        })
    }

    fn any_non_empty(&mut self) -> bool {
        self.lines.any(|line| !line.trim().is_empty())
    }
}

fn protocol_text(value: &str, label: &str, allow_empty: bool) -> Result<String, String> {
    if (!allow_empty && value.is_empty()) || value.len() > 256 {
        return Err(format!("{label} inválido: debe contener de 1 a 256 bytes"));
    }
    if value.chars().any(char::is_control) {
        return Err(format!("{label} contiene caracteres de control"));
    }
    Ok(value.to_string())
}

fn optional_protocol_text(value: &str, label: &str) -> Result<Option<String>, String> {
    if value.is_empty() {
        Ok(None)
    } else {
        protocol_text(value, label, false).map(Some)
    }
}

fn escape_st_protocol_text(value: &str) -> Result<String, String> {
    if value.len() > 256 || value.chars().any(char::is_control) {
        return Err(
            "los nombres exportados a ST no pueden contener controles ni superar 256 bytes".into(),
        );
    }
    Ok(value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn st_float(value: f32) -> String {
    let mut text = value.to_string();
    if !text.contains(['.', 'e', 'E']) {
        text.push_str(".0");
    }
    text
}

fn resolve_compiler_path(compiler: &Path) -> Result<PathBuf, String> {
    if compiler.as_os_str().is_empty() {
        return Err("configura la ruta de pyforgec".into());
    }
    if compiler.is_absolute() || compiler.components().count() > 1 {
        let path = if compiler.is_absolute() {
            compiler.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|error| format!("obtener carpeta actual: {error}"))?
                .join(compiler)
        };
        if !path.is_file() {
            return Err(format!("no se encontró pyforgec en {}", path.display()));
        }
        Ok(path)
    } else {
        // Un nombre simple se deja para que `Command` lo resuelva mediante PATH.
        Ok(compiler.to_path_buf())
    }
}

fn temporary_binary_path() -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let extension = if cfg!(windows) { ".exe" } else { "" };
    std::env::temp_dir().join(format!(
        "pyforge-editor3d-{}-{stamp}{extension}",
        std::process::id()
    ))
}

struct ProcessOutput {
    status: ExitStatus,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn run_limited(mut command: Command, timeout: Duration) -> Result<ProcessOutput, String> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("iniciar proceso: {error}"))?;
    let stdout = child.stdout.take().ok_or("stdout no disponible")?;
    let stderr = child.stderr.take().ok_or("stderr no disponible")?;
    let stdout_reader = thread::spawn(move || read_capped(stdout, MAX_STDOUT_BYTES));
    let stderr_reader = thread::spawn(move || read_capped(stderr, MAX_STDERR_BYTES));
    let started = Instant::now();
    let status = loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("esperar proceso: {error}"))?
        {
            break status;
        }
        if started.elapsed() >= timeout {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_reader.join();
            let _ = stderr_reader.join();
            return Err(format!("el proceso excedió {} segundos", timeout.as_secs()));
        }
        thread::sleep(Duration::from_millis(20));
    };
    let stdout = stdout_reader
        .join()
        .map_err(|_| "falló el lector de stdout".to_string())??;
    let stderr = stderr_reader
        .join()
        .map_err(|_| "falló el lector de stderr".to_string())??;
    Ok(ProcessOutput {
        status,
        stdout,
        stderr,
    })
}

fn read_capped(mut reader: impl Read, maximum: usize) -> Result<Vec<u8>, String> {
    let mut result = Vec::new();
    let mut buffer = [0_u8; 8 * 1024];
    loop {
        let count = reader
            .read(&mut buffer)
            .map_err(|error| format!("leer salida del proceso: {error}"))?;
        if count == 0 {
            return Ok(result);
        }
        if result.len() + count > maximum {
            return Err(format!("la salida del proceso supera {maximum} bytes"));
        }
        result.extend_from_slice(&buffer[..count]);
    }
}

fn format_process_error(action: &str, output: &ProcessOutput) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let detail = if stderr.trim().is_empty() {
        stdout.trim()
    } else {
        stderr.trim()
    };
    if detail.is_empty() {
        format!("no se pudo {action}: estado {}", output.status)
    } else {
        format!("no se pudo {action}: {detail}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_bounded_scene() {
        let text = concat!(
            "PYFORGE_EDITOR3D_V1\n",
            "Nivel\n1\n",
            "Jugador\ncubo\njugador\n",
            "1\n2\n3\n0\n0\n0\n1\n1\n2\n1\n",
            "FIN_PYFORGE_EDITOR3D_V1\n"
        );
        let scene = parse_scene_protocol(text).expect("protocolo válido");
        assert_eq!(scene.name, "Nivel");
        assert_eq!(scene.entities[0].transform.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn rejects_incomplete_or_non_finite_transforms() {
        let text = concat!(
            "PYFORGE_EDITOR3D_V1\nNivel\n1\nCubo\ncubo\nbase\n",
            "0\n0\n0\n0\n0\n0\nNaN\n1\n1\n1\nFIN_PYFORGE_EDITOR3D_V1\n"
        );
        assert!(parse_scene_protocol(text)
            .unwrap_err()
            .contains("no es finita"));
        assert!(parse_scene_protocol("PYFORGE_EDITOR3D_V1\n").is_err());
    }

    #[test]
    fn exports_valid_st_calls() {
        let scene = Scene {
            version: SCENE_VERSION,
            name: "Demo".into(),
            entities: vec![Entity {
                name: "Cubo \\\"A\\\"".into(),
                mesh: Some("cubo".into()),
                material: Some("cristal".into()),
                transform: Transform::default(),
            }],
        };
        let source = scene_to_st(&scene).expect("exportación válida");
        assert!(source.contains("importar \"editor3d\";"));
        assert!(source.contains("editor3d.entidad"));
        assert!(source.contains("editor3d.rotacion"));
    }
}
