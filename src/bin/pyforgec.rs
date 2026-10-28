//! Ejecutable de línea de comandos para archivos Python individuales.
//! Interpreta opciones y usa la biblioteca pyforge::driver para comprobar, mostrar IR,
//! generar ensamblador, compilar o ejecutar. No implementa aquí el lenguaje.
use pyforge::{
    driver::{build_executable, compile_file, default_output},
    VERSION,
};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{exit, Command},
};
/// Imprime los comandos y opciones disponibles en la salida estándar.
/// No necesita archivos ni ejecuta operaciones de compilación.
fn usage() {
    println!("pyforgec {VERSION}\n\nUSO:\n  pyforgec check <archivo.py|archivo.st>\n  pyforgec emit-ir <archivo.py|archivo.st> [-o file.ir] [--no-opt]\n  pyforgec emit-llvm <archivo.py|archivo.st> [-o file.ll] [--no-opt]\n  pyforgec emit-asm <archivo.py|archivo.st> [-o file.s] [--no-opt]\n  pyforgec build <archivo.py|archivo.st> [-o executable] [--no-opt] [--keep]\n  pyforgec run <archivo.py|archivo.st> [--no-opt] [--keep]\n  pyforgec language-json\n  pyforgec --version\n\nBACKEND ÚNICO:\n  LLVM 21 mediante Inkwell");
}
/// Punto de entrada del proceso. Delega en real; ante Err escribe el mensaje
/// en stderr y termina con código 1. El éxito termina normalmente.
fn main() {
    if let Err(e) = real() {
        eprintln!("pyforgec: error: {e}");
        exit(1)
    }
}
/// Lee argumentos del proceso y selecciona archivo, -o, --no-opt y --keep.
/// Check compila en memoria; emit-ir/emit-asm muestran o guardan textos; build
/// crea el ejecutable y run compila a la ruta predeterminada y lo ejecuta.
/// Crea directorios cuando corresponde y propaga errores como String.
/// El análisis de opciones no rechaza todas las opciones desconocidas.
fn real() -> Result<(), String> {
    let a: Vec<String> = env::args().skip(1).collect();
    if a.is_empty() {
        usage();
        return Ok(());
    }
    if a[0] == "--version" || a[0] == "-V" {
        println!("pyforgec {VERSION}");
        return Ok(());
    }
    if a[0] == "language-json" {
        // Protocolo pequeño y estable para IDEs: las tablas proceden del mismo
        // binario que compila, así no se duplica manualmente KEYWORDS.
        let mut words: Vec<_> = pyforge::st::lexer::KEYWORDS
            .iter()
            .map(|(word, _)| *word)
            .chain(
                pyforge::st::lexer::CONTEXTUAL_WORDS
                    .iter()
                    .map(|(word, _)| *word),
            )
            .collect();
        words.sort_unstable();
        words.dedup();
        print!("{{\"version\":\"{VERSION}\",\"keywords\":[");
        for (index, word) in words.iter().enumerate() {
            if index != 0 {
                print!(",");
            }
            print!("\"{word}\"");
        }
        println!("]}}");
        return Ok(());
    }
    if a.len() < 2 {
        return Err("falta el archivo de entrada".into());
    }
    let cmd = &a[0];
    let input = Path::new(&a[1]);
    let opt = !a.iter().any(|x| x == "--no-opt");
    let keep = a.iter().any(|x| x == "--keep");
    reject_removed_backend(&a)?;
    let out_arg = a
        .windows(2)
        .find(|w| w[0] == "-o")
        .map(|w| PathBuf::from(&w[1]));
    match cmd.as_str() {
        "check" => {
            let _ = compile_file(input, opt)?;
            println!("OK: {} [llvm]", input.display());
        }
        "emit-ir" => {
            let art = compile_file(input, opt)?;
            if let Some(p) = out_arg {
                fs::write(&p, art.ir).map_err(|e| e.to_string())?;
                println!("IR: {}", p.display());
            } else {
                print!("{}", art.ir);
            }
        }
        "emit-llvm" => {
            let llvm_ir = compile_file(input, opt)?.llvm_ir;
            if let Some(p) = out_arg {
                fs::write(&p, llvm_ir).map_err(|e| e.to_string())?;
                println!("LLVM IR: {}", p.display());
            } else {
                print!("{llvm_ir}");
            }
        }
        "emit-asm" => {
            let art = compile_file(input, opt)?;
            if let Some(p) = out_arg {
                fs::write(&p, art.asm).map_err(|e| e.to_string())?;
                println!("ASM: {}", p.display());
            } else {
                print!("{}", art.asm);
            }
        }
        "build" => {
            let out = out_arg.unwrap_or_else(|| default_output(input));
            if let Some(d) = out.parent() {
                fs::create_dir_all(d).map_err(|e| e.to_string())?;
            }
            build_executable(input, &out, opt, keep)?;
            println!("Compilado: {} [llvm]", out.display());
        }
        "run" => {
            let out = default_output(input);
            if let Some(d) = out.parent() {
                fs::create_dir_all(d).map_err(|e| e.to_string())?;
            }
            build_executable(input, &out, opt, keep)?;
            let st = Command::new(&out)
                .status()
                .map_err(|e| format!("no se pudo ejecutar {}: {e}", out.display()))?;
            if !st.success() {
                return Err(format!("el programa terminó con estado {st}"));
            }
        }
        _ => {
            usage();
            return Err(format!("comando desconocido: {cmd}"));
        }
    }
    Ok(())
}

/// Rechaza la opción anterior para impedir que una orden antigua parezca elegir
/// otro generador. LLVM es obligatorio y no necesita selector.
fn reject_removed_backend(args: &[String]) -> Result<(), String> {
    if args.iter().any(|argument| argument == "--backend") {
        Err("--backend fue retirado: PyForge compila siempre con LLVM 21".into())
    } else {
        Ok(())
    }
}
