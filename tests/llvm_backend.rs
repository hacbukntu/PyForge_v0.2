//! Pruebas de ejecución del backend LLVM obligatorio. Verifican el ELF final, no
//! solo que Inkwell acepte el módulo, y comparan las dos rutas de optimización.

use pyforge::driver::build_executable;
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Temp(PathBuf);

impl Temp {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "pyforge-llvm-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).expect("crear temporal LLVM");
        Self(path)
    }
}

impl Drop for Temp {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn llvm_program(source: &str, expected: &str) {
    let temp = Temp::new();
    let input = temp.0.join("main.st");
    fs::write(&input, source).expect("escribir fuente ST");
    for optimize in [false, true] {
        let output = temp.0.join(if optimize { "optimizado" } else { "directo" });
        build_executable(&input, &output, optimize, false).expect("construir con LLVM");
        assert_eq!(&fs::read(&output).expect("leer ELF")[..4], b"\x7fELF");
        let result = Command::new(&output).output().expect("ejecutar ELF LLVM");
        assert!(result.status.success(), "{:?}", result.status);
        assert_eq!(String::from_utf8(result.stdout).unwrap(), expected);
        assert!(result.stderr.is_empty());
    }
}

#[test]
fn control_funciones_y_colecciones_se_ejecutan_con_llvm() {
    llvm_program(
        r#"
funcion sumar(a: entero; b: entero) -> entero { retornar a + b; }
variable valores: lista = [2, 3, 4];
variable total: entero = 0;
para i en rango(longitud(valores)) {
    total += valores[i];
}
mostrar(sumar(total; 1));
"#,
        "10\n",
    );
}

#[test]
fn excepciones_conservan_la_salida_en_el_backend_llvm() {
    llvm_program(
        r#"
intentar {
    lanzar "fallo controlado";
} capturar error {
    mostrar(error);
} finalmente {
    mostrar("fin");
}
"#,
        "fallo controlado\nfin\n",
    );
}
