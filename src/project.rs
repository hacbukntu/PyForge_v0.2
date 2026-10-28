//! Creación y configuración de proyectos para bin/pyforge.
//! new_project escribe la estructura inicial; project_entry obtiene nombre y entrada.
//! El gestor pasa esa entrada a driver, que realiza la compilación real.
use std::{
    fs,
    path::{Path, PathBuf},
};

/// Crea el directorio `name` con PyForge.toml, src/main.st y .gitignore.
/// Valida caracteres ASCII y rechaza rutas existentes. Devuelve () o error.
/// La creación no es transaccional: un fallo puede dejar archivos parciales.
pub fn new_project(name: &str) -> Result<(), String> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err("el nombre del proyecto solo admite letras, dígitos, '_' y '-'".into());
    }
    let root = Path::new(name);
    if root.exists() {
        return Err(format!("la ruta ya existe: {}", root.display()));
    }
    fs::create_dir_all(root.join("src")).map_err(|e| e.to_string())?;
    fs::write(root.join("PyForge.toml"),format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n\n[build]\nentry = \"src/main.st\"\noptimize = true\n")).map_err(|e|e.to_string())?;
    fs::write(root.join("src/main.st"),"// Función de ejemplo del lenguaje ST.\nfuncion sumar(a: entero, b: entero) -> entero {\n    retornar a + b;\n}\n\nmostrar(\"Hola desde ST\");\nmostrar(sumar(20, 22));\n").map_err(|e|e.to_string())?;
    fs::write(root.join(".gitignore"), "target/\ndist/\n.pyforge-build/\n")
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Lee PyForge.toml bajo `root` y devuelve (nombre, ruta de entrada).
/// Si faltan valores, usa el nombre del directorio o app y src/main.py.
/// Solo consulta name y entry; no interpreta optimize ni todo el formato TOML.
pub fn project_entry(root: &Path) -> Result<(String, PathBuf), String> {
    let cfg = fs::read_to_string(root.join("PyForge.toml")).map_err(|_| {
        "no se encontró PyForge.toml; ejecuta el comando dentro de un proyecto".to_string()
    })?;
    let name = find_value(&cfg, "name").unwrap_or_else(|| {
        root.file_name()
            .and_then(|x| x.to_str())
            .unwrap_or("app")
            .to_string()
    });
    let entry = find_value(&cfg, "entry").unwrap_or_else(|| "src/main.py".into());
    Ok((name, root.join(entry)))
}
/// Busca en `s` la primera línea que empiece por `key` tras recortar espacios.
/// Devuelve lo situado después de =, sin espacios ni comillas exteriores, o None.
/// No distingue secciones TOML, prefijos de claves ni comentarios al final.
fn find_value(s: &str, key: &str) -> Option<String> {
    for line in s.lines() {
        let l = line.trim();
        if l.starts_with(key) {
            if let Some(eq) = l.find('=') {
                return Some(l[eq + 1..].trim().trim_matches('"').to_string());
            }
        }
    }
    None
}
