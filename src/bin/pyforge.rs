//! Ejecutable de línea de comandos para gestionar proyectos.
//! Usa project para crear o leer PyForge.toml y driver para compilar la entrada.
//! Selecciona el directorio de salida, ejecuta el resultado o limpia los artefactos.
use pyforge::{
    driver::{build_executable, compile_file},
    project::{new_project, project_entry},
    VERSION,
};
use std::{
    env, fs,
    path::Path,
    process::{exit, Command},
};
/// Imprime los comandos y opciones disponibles en la salida estándar.
/// No necesita archivos ni ejecuta operaciones de compilación.
fn usage() {
    println!("pyforge {VERSION}\n\nUSO:\n  pyforge new <name> [--web]\n  pyforge web [--port <puerto>]\n  pyforge motor3d [--vulkan|--opengl|--editor]\n  pyforge motor3d --juego <binario-st>\n  pyforge motor3d --juego <perfil> <binario-st>\n  pyforge check\n  pyforge build [--no-opt] [--keep]\n  pyforge run [--no-opt] [--keep]\n  pyforge clean\n  pyforge --version\n\nCompilador: LLVM 21 mediante Inkwell\nPerfiles 3D: portales, cristales, laberinto, guardianes\nArchivo del proyecto: PyForge.toml");
}
/// Punto de entrada del proceso. Delega en real; ante Err escribe el mensaje
/// en stderr y termina con código 1. El éxito termina normalmente.
fn main() {
    if let Err(e) = real() {
        eprintln!("pyforge: error: {e}");
        exit(1)
    }
}
/// Lee argumentos y resuelve ayuda, versión o creación de proyecto.
/// Para los demás comandos obtiene nombre y entrada mediante project_entry.
/// --no-opt selecciona target/debug; por defecto usa target/release.
/// Check valida, build compila, run compila y ejecuta; --keep conserva intermedios.
/// Clean borra target/dist del directorio actual e ignora errores de borrado.
/// Devuelve () o un diagnóstico; no busca manifiestos en directorios ascendentes.
fn real() -> Result<(), String> {
    let a: Vec<String> = env::args().skip(1).collect();
    if a.is_empty() || a[0] == "--help" || a[0] == "-h" {
        usage();
        return Ok(());
    }
    if a[0] == "--version" || a[0] == "-V" {
        println!("pyforge {VERSION}");
        return Ok(());
    }
    if a[0] == "new" {
        let n = a.get(1).ok_or("falta el nombre del proyecto")?;
        if a.iter().any(|x| x == "--web") {
            pyforge::web_project::new_web_project(n)?;
        } else {
            new_project(n)?;
        }
        println!("Proyecto PyForge creado: {n}");
        return Ok(());
    }
    if a[0] == "web" {
        return web(&a[1..]);
    }
    if a[0] == "motor3d" {
        return motor3d(&a[1..]);
    }
    let root = Path::new(".");
    let (name, entry) = project_entry(root)?;
    let opt = !a.iter().any(|x| x == "--no-opt");
    let keep = a.iter().any(|x| x == "--keep");
    reject_removed_backend(&a)?;
    let out = Path::new("target")
        .join(if opt { "release" } else { "debug" })
        .join(&name);
    match a[0].as_str() {
        "check" => {
            let _ = compile_file(&entry, opt)?;
            println!("OK: {} [llvm]", entry.display());
        }
        "build" => {
            if let Some(d) = out.parent() {
                fs::create_dir_all(d).map_err(|e| e.to_string())?;
            }
            build_executable(&entry, &out, opt, keep)?;
            println!("Compilado: {} [llvm]", out.display());
        }
        "run" => {
            if let Some(d) = out.parent() {
                fs::create_dir_all(d).map_err(|e| e.to_string())?;
            }
            build_executable(&entry, &out, opt, keep)?;
            let st = Command::new(&out).status().map_err(|e| e.to_string())?;
            if !st.success() {
                return Err(format!("el programa terminó con estado {st}"));
            }
        }
        "clean" => {
            let _ = fs::remove_dir_all("target");
            let _ = fs::remove_dir_all("dist");
            println!("Se limpiaron los archivos de compilación.");
        }
        _ => {
            usage();
            return Err(format!("comando desconocido: {}", a[0]));
        }
    }
    Ok(())
}

/// Rechaza órdenes antiguas con selector: LLVM es el único generador público.
fn reject_removed_backend(args: &[String]) -> Result<(), String> {
    if args.iter().any(|argument| argument == "--backend") {
        Err("--backend fue retirado: PyForge compila siempre con LLVM 21".into())
    } else {
        Ok(())
    }
}

/// Arranca el componente gráfico instalado sin cargarlo dentro del compilador.
fn motor3d(args: &[String]) -> Result<(), String> {
    if args.first().is_some_and(|arg| arg == "--juego") {
        if args.len() != 2 && args.len() != 3 {
            return Err("uso: pyforge motor3d --juego [perfil] <binario-st>".into());
        }
        let command = "pyforge-game3d";
        let mut process = Command::new(command);
        if args.len() == 3 {
            process.args(["--perfil", &args[1], &args[2]]);
        } else {
            process.arg(&args[1]);
        }
        let status = process
            .status()
            .map_err(|e| format!("no se pudo iniciar {command}: {e}; instala el componente 3D"))?;
        if !status.success() {
            return Err(format!("{command} terminó con estado {status}"));
        }
        return Ok(());
    }
    if args.len() > 1 {
        return Err("uso: pyforge motor3d [--vulkan|--opengl|--editor]".into());
    }
    let editor = args.first().is_some_and(|x| x == "--editor");
    if let Some(arg) = args.first() {
        if !editor && arg != "--vulkan" && arg != "--opengl" {
            return Err("backend desconocido: usa --vulkan, --opengl o --editor".into());
        }
    }
    let command = if editor {
        "pyforge-editor3d"
    } else {
        "pyforge-engine3d"
    };
    let mut child = Command::new(command);
    if !editor {
        child.args(args);
    }
    let status = child
        .status()
        .map_err(|e| format!("no se pudo iniciar {command}: {e}; instala el componente 3D"))?;
    if !status.success() {
        return Err(format!("{command} terminó con estado {status}"));
    }
    Ok(())
}

/// Compila las vistas ST y arranca Django solo en loopback; producción usa WSGI + HTTPS.
fn web(args: &[String]) -> Result<(), String> {
    let port = if args.is_empty() {
        8000
    } else if args.len() == 2 && args[0] == "--port" {
        args[1].parse::<u16>().map_err(|_| "puerto inválido")?
    } else {
        return Err("uso: pyforge web [--port <puerto>]".into());
    };
    if port == 0 {
        return Err("el puerto debe estar entre 1 y 65535".into());
    }
    if !Path::new("web/manage.py").is_file() {
        return Err("crea un proyecto con pyforge new nombre --web".into());
    }
    let (name, entry) = project_entry(Path::new("."))?;
    let out = Path::new("target/release").join(name);
    fs::create_dir_all("target/release").map_err(|e| e.to_string())?;
    build_executable(&entry, &out, true, false)?;
    let binary = out.canonicalize().map_err(|e| e.to_string())?;
    let python = env::var("PYFORGE_WEB_PYTHON").unwrap_or_else(|_| "python3".into());
    let status = Command::new(&python)
        .args(["web/manage.py", "migrate", "--noinput"])
        .env("ST_WEB_LOCAL", "1")
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err(
            "instala web/requirements.txt en PYFORGE_WEB_PYTHON y revisa migraciones".into(),
        );
    }
    let status = Command::new(&python)
        .args([
            "web/manage.py",
            "runserver",
            &format!("127.0.0.1:{port}"),
            "--noreload",
            "--insecure",
        ])
        .env("ST_WEB_LOCAL", "1")
        .env("ST_WEB_BINARY", binary)
        .status()
        .map_err(|e| e.to_string())?;
    if !status.success() {
        return Err("el servidor web terminó con error".into());
    }
    Ok(())
}
