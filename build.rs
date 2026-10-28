//! Construye una biblioteca nativa autocontenida y la incrusta en el compilador.
//! El usuario final necesita binutils, no Rust ni Python, para enlazar este módulo.
use std::{env, path::PathBuf, process::Command};

/// Cargo vuelve a generar el archivo cuando cambia la implementación o el destino.
fn main() {
    println!("cargo:rerun-if-changed=runtime/native/lib.rs");
    let target = env::var("TARGET").expect("TARGET de Cargo");
    let output = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR de Cargo"));
    let archive = output.join("libpyforge_native.a");
    if !target.starts_with("x86_64-unknown-linux-") {
        // El frontend puede construirse en otros sistemas; este backend no puede ejecutarse allí.
        std::fs::write(archive, []).expect("marcador de runtime no disponible");
        return;
    }
    let status = Command::new(env::var_os("RUSTC").unwrap_or_else(|| "rustc".into()))
        .args([
            "runtime/native/lib.rs",
            "--crate-name",
            "pyforge_native",
            "--crate-type",
            "staticlib",
            "--edition=2021",
            "--target",
        ])
        .arg(target)
        .args([
            "-C",
            "opt-level=2",
            "-C",
            "panic=abort",
            "-C",
            "relocation-model=static",
            "-C",
            "no-redzone=yes",
            "-o",
        ])
        .arg(archive)
        .status()
        .expect("ejecutar rustc para el runtime nativo");
    assert!(
        status.success(),
        "no se pudo construir el runtime nativo de ST"
    );
}
