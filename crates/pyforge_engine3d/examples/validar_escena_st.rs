//! Verifica el puente ST del editor sin abrir una ventana.
//!
//! Resulta útil en integración continua y al crear escenas grandes: compila el
//! archivo, ejecuta su protocolo acotado y muestra el resumen validado.

use pyforge_engine3d::st_scene::compile_and_run_scene_script;
use std::{env, path::Path};

fn main() {
    if let Err(error) = run() {
        eprintln!("validar_escena_st: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut arguments = env::args().skip(1);
    let compiler = arguments
        .next()
        .ok_or("uso: validar_escena_st <pyforgec> <escena.st>")?;
    let source = arguments
        .next()
        .ok_or("uso: validar_escena_st <pyforgec> <escena.st>")?;
    if arguments.next().is_some() {
        return Err("se recibieron argumentos adicionales".into());
    }
    let scene = compile_and_run_scene_script(Path::new(&compiler), Path::new(&source))?;
    println!(
        "Escena ST válida: {} ({} entidades)",
        scene.name,
        scene.entities.len()
    );
    Ok(())
}
