//! Coordinación de la compilación usada por ambos binarios.
//! compile_text conecta parser, semantic, lower, optimizer y codegen sin escribir archivos.
//! compile_file añade lectura del archivo; build_executable añade escritura, as y ld.
//! CompileArtifacts permite a las interfaces mostrar la IR o el ensamblador.
use crate::{
    codegen::llvm::emit_linux_artifacts, ir::format_ir, lower::lower, optimizer::optimize,
    parser::parse_source, semantic::analyze,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

/// Resultados en memoria: asm es ensamblador e ir es IR textual.
/// Ninguna de estas cadenas es todavía un ejecutable ELF.
pub struct CompileArtifacts {
    pub asm: String,
    pub ir: String,
    /// LLVM IR optimizada que produce el único backend del compilador.
    pub llvm_ir: String,
    /// Indica que las excepciones conservaron temporalmente su ABI ensambladora.
    pub llvm_compatibility: bool,
}

/// Compila `src` en memoria: parse_source → analyze → lower → optimización opcional
/// → verificación/emisión LLVM. `opt` activa los optimizadores ST y LLVM.
pub fn compile_text(src: &str, opt: bool) -> Result<CompileArtifacts, String> {
    compile_program(parse_source(src)?, opt)
}

/// Compila texto ST mediante su analizador independiente y las etapas comunes.
/// No altera compile_text, que continúa interpretando únicamente Python.
pub fn compile_st_text(src: &str, opt: bool) -> Result<CompileArtifacts, String> {
    compile_program(crate::st::parse_source(src)?, opt)
}

/// Recibe el AST de cualquiera de los lenguajes y aplica las mismas comprobaciones,
/// optimizaciones y generación nativa. Evita duplicar la lógica del compilador.
fn compile_program(ast: crate::ast::Program, opt: bool) -> Result<CompileArtifacts, String> {
    let sem = analyze(&ast).map_err(|e| format!("análisis semántico: {e}"))?;
    let mut ir = lower(&ast, &sem).map_err(|e| format!("conversión a IR: {e}"))?;
    if opt {
        optimize(&mut ir);
    }
    let ir_text = format_ir(&ir);
    let artifacts =
        emit_linux_artifacts(&ir, opt).map_err(|error| format!("generación LLVM: {error}"))?;
    Ok(CompileArtifacts {
        asm: artifacts.assembly,
        ir: ir_text,
        llvm_ir: artifacts.llvm_ir,
        llvm_compatibility: artifacts.compatibility_mode,
    })
}
/// Lee `path` como UTF-8: la extensión .st selecciona ST; el resto conserva Python.
/// Devuelve artefactos en memoria o un error de lectura/compilación.
pub fn compile_file(path: &Path, opt: bool) -> Result<CompileArtifacts, String> {
    let src = fs::read_to_string(path)
        .map_err(|e| format!("no se puede leer {}: {e}", path.display()))?;
    if path.extension().and_then(|x| x.to_str()) == Some("st") {
        compile_program(crate::st::modules::load(path)?, opt)
            .map_err(|e| format!("entrada {}: {e}", path.display()))
    } else {
        compile_program(parse_source(&src)?, opt).map_err(|e| format!("{}: {e}", path.display()))
    }
}

/// Compila `input` y genera el ejecutable `output`; `opt` controla optimización
/// y `keep` conserva .s/.o. Crea .pyforge-build junto a la salida, escribe ensamblador,
/// ejecuta as --64 y luego ld. Propaga fallos y limpia intermedios tras el éxito
/// si no se pidió conservarlos; ignora errores de limpieza. Comprueba Linux,
/// pero no la CPU. Compilaciones a la misma salida comparten nombres intermedios.
pub fn build_executable(input: &Path, output: &Path, opt: bool, keep: bool) -> Result<(), String> {
    if !cfg!(target_os = "linux") {
        return Err("el generador nativo solo admite Linux x86-64".into());
    }
    // Compila antes de escribir intermedios junto a la ruta de salida.
    let art = compile_file(input, opt)?;
    let build_dir = output
        .parent()
        .unwrap_or(Path::new("."))
        .join(".pyforge-build");
    fs::create_dir_all(&build_dir).map_err(|e| e.to_string())?;
    let stem = output.file_name().and_then(|x| x.to_str()).unwrap_or("app");
    let asm_path = build_dir.join(format!("{stem}.s"));
    let obj_path = build_dir.join(format!("{stem}.o"));
    fs::write(&asm_path, &art.asm).map_err(|e| e.to_string())?;
    // as convierte el texto ensamblador a objeto; un estado no exitoso detiene el proceso.
    let as_status = Command::new("as")
        .args(["--64", "-o"])
        .arg(&obj_path)
        .arg(&asm_path)
        .status()
        .map_err(|e| format!("no se pudo ejecutar GNU as: {e}. Instala binutils."))?;
    if !as_status.success() {
        return Err(
            "falló el ensamblador; usa 'pyforgec emit-asm' para inspeccionar la salida".into(),
        );
    }
    // ld enlaza el objeto para producir el ejecutable ELF.
    let mut linker = Command::new("ld");
    linker.arg("-o").arg(output).arg(&obj_path);
    // Solo los programas que llaman a la ABI 1 incorporan la biblioteca Rust.
    // El archivo viene incrustado: la instalación no necesita rustc ni fuentes externas.
    let native_path = build_dir.join(format!("{stem}.native.a"));
    let needs_native = art.asm.lines().any(|line| {
        let instruction = line.trim_start();
        instruction.starts_with("call") && instruction.contains("pf_native_")
    });
    if needs_native {
        let archive = include_bytes!(concat!(env!("OUT_DIR"), "/libpyforge_native.a"));
        if archive.is_empty() {
            return Err("runtime nativo disponible solo para Linux x86-64".into());
        }
        fs::write(&native_path, archive).map_err(|e| e.to_string())?;
        linker.arg(&native_path).arg("--gc-sections");
    }
    let ld_status = linker
        .status()
        .map_err(|e| format!("no se pudo ejecutar GNU ld: {e}. Instala binutils."))?;
    if !ld_status.success() {
        return Err("falló el enlazador".into());
    }
    // Limpia solamente tras enlazar correctamente, si no se pidieron intermedios.
    if !keep {
        if needs_native {
            let _ = fs::remove_file(&native_path);
        }
        let _ = fs::remove_file(&asm_path);
        let _ = fs::remove_file(&obj_path);
        let _ = fs::remove_dir(&build_dir);
    }
    Ok(())
}

/// Calcula `dist/<nombre sin extensión>` para `input`, o usa a.out si falta
/// un nombre UTF-8 válido. Devuelve una ruta sin crear archivos ni directorios.
pub fn default_output(input: &Path) -> PathBuf {
    let stem = input
        .file_stem()
        .and_then(|x| x.to_str())
        .unwrap_or("a.out");
    PathBuf::from("dist").join(stem)
}
