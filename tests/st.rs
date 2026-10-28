//! Pruebas de regresión del nuevo lenguaje y de su integración con Python.
//! Los ejecutables y proyectos se crean en directorios temporales propios.
use pyforge::driver::{compile_file, compile_st_text, compile_text};
use pyforge::st::lexer::{tokenize, Keyword, Kind, KEYWORDS};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::atomic::{AtomicUsize, Ordering},
};

/// Contador que evita colisiones entre pruebas paralelas del mismo proceso.
static NEXT: AtomicUsize = AtomicUsize::new(0);
/// Directorio temporal aislado; al destruirlo elimina solamente sus propios archivos.
struct Temp(PathBuf);
impl Temp {
    /// Reserva un nombre único por proceso/prueba y crea el directorio.
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "pyforge-st-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temp {
    /// Limpia artefactos temporales tanto al pasar como al fallar una prueba.
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Construye mediante el CLI y ejecuta un caso ST con salida esperada explícita.
/// Repite con y sin optimización para detectar diferencias de integración.
fn native(source: &str, expected: &str) {
    let temp = Temp::new();
    let input = temp.0.join("programa.st");
    let output = temp.0.join("programa");
    fs::write(&input, source).unwrap();
    for flags in [vec![], vec!["--no-opt"]] {
        let result = Command::new(env!("CARGO_BIN_EXE_pyforgec"))
            .arg("build")
            .arg(&input)
            .arg("-o")
            .arg(&output)
            .args(flags)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(&fs::read(&output).unwrap()[..4], b"\x7fELF");
        let result = Command::new(&output).output().unwrap();
        assert!(result.status.success());
        assert_eq!(String::from_utf8(result.stdout).unwrap(), expected);
    }
}

/// Las palabras solo son reservadas como tokens completos, nunca dentro de cadenas/comentarios.
#[test]
fn palabras_y_cadenas() {
    let tokens = tokenize("si siempre \"si { // 😀\" // funcion\n # variable\n verdadero").unwrap();
    assert_eq!(tokens[0].kind, Kind::Keyword(Keyword::Si));
    assert_eq!(tokens[1].kind, Kind::Ident("siempre".into()));
    assert_eq!(tokens[2].kind, Kind::Str("si { // 😀".into()));
    assert_eq!(tokens[3].kind, Kind::Keyword(Keyword::Verdadero));
    assert_eq!(tokens[3].line, 3);
    let mut words = std::collections::HashSet::new();
    for (word, _) in KEYWORDS {
        assert!(words.insert(word), "palabra duplicada: {word}");
    }
    assert!(KEYWORDS.iter().any(|(word, _)| *word == "publica"));
    assert!(KEYWORDS.iter().any(|(word, _)| *word == "privada"));
    assert!(KEYWORDS.iter().any(|(word, _)| *word == "mixto"));
}

/// ST diferencia declarar y reasignar: no crea nombres por un error ortográfico.
#[test]
fn declaraciones_seguras_y_tipos_estables() {
    assert!(compile_st_text("total = 1;", true)
        .err()
        .unwrap()
        .contains("variable no declarada"));
    assert!(
        compile_st_text("variable total = 1; variable total = 2;", true)
            .err()
            .unwrap()
            .contains("variable ya declarada")
    );
    assert!(compile_st_text("variable total = 1; total = 2; mostrar(total);", true).is_ok());
    assert!(compile_st_text("variable total = 1; total = 2.0;", true)
        .err()
        .unwrap()
        .contains("cambia de tipo"));
    assert!(compile_st_text("publica variable x = 1;", true).is_ok());
    assert!(compile_st_text("funcion f() { publica variable x = 1; }", true).is_err());
}

/// Una entrada inválida debe devolver un error, sin panic ni aceptación silenciosa.
#[test]
fn errores_sintacticos() {
    for source in [
        "sino {}",
        "}",
        "si verdadero {",
        "mostrar(1)",
        "mostrar();",
        "variable si = 1;",
        "variable x: nada = 1;",
        "retornar 1;",
        "funcion f(x: entero, x: entero) {}",
        "funcion f() { funcion g() {} }",
        "mostrar(1 < 2 < 3);",
        "mostrar(\"sin cierre);",
        "mostrar(@);",
        "funcion __main__() {}",
        "variable print = 1;",
        "para i en rango(0, 2, 1) {}",
        "funcion f(a: entero,b: entero,c: entero,d: entero,e: entero,f: entero,g: entero) {}",
    ] {
        let error = pyforge::st::parse_source(source).expect_err(source);
        assert!(error.starts_with("ST línea "), "{source}: {error}");
    }
    let error = tokenize("\n  @").unwrap_err();
    assert!(error.contains("línea 2, columna 3"));
}

/// ST reutiliza las comprobaciones semánticas: tipos, nombres y control de bucles.
#[test]
fn errores_semanticos() {
    for source in [
        "mostrar(desconocida);",
        "variable x: booleano = 1;",
        "romper;",
        "continuar;",
        "variable x = 1; x = verdadero;",
        "mostrar(inexistente());",
        "funcion f(x: entero) -> entero { retornar x; } mostrar(f());",
    ] {
        assert!(compile_st_text(source, true).is_err(), "{source}");
    }
}

/// La API Python permanece separada; la extensión .st activa únicamente la nueva API.
#[test]
fn seleccion_por_extension() {
    assert!(compile_text("print(42)\n", true).is_ok());
    assert!(compile_text("mostrar(42);", true).is_err());
    assert!(compile_st_text("print(42)\n", true).is_err());
    let temp = Temp::new();
    for name in ["main.st", "main.py"] {
        let path = temp.0.join(name);
        fs::write(&path, "mostrar(42);").unwrap();
        assert_eq!(compile_file(&path, true).is_ok(), name.ends_with(".st"));
    }
}

/// Comprueba que las expresiones se estructuran por precedencia antes de optimizar.
#[test]
fn precedencia_y_operadores() {
    native("mostrar(2 + 3 * 4); mostrar((2 + 3) * 4); mostrar(9 / 2); mostrar(9 % 2); mostrar(no 1 == 2); mostrar(verdadero o falso y falso); variable x = 10; x += 2; x -= 1; x *= 2; x /= 2; x %= 4; mostrar(x);",
        "14\n20\n4\n1\nTrue\nTrue\n3\n");
}

/// Verifica cortocircuito observable: la función con impresión no debe ejecutarse.
#[test]
fn cortocircuito() {
    native("funcion efecto() -> booleano { mostrar(999); retornar verdadero; } mostrar(falso y efecto()); mostrar(verdadero o efecto());", "False\nTrue\n");
}

/// Si el archivo define una función `main`, debe ejecutarse antes del cuerpo superior.
#[test]
fn main_se_ejecuta_primero() {
    native("funcion main() { mostrar(1); } mostrar(2);", "1\n2\n");
}

/// El compilador Python oficial debe aceptar listas, diccionarios, índices y None.
#[test]
fn python_oficial_basico() {
    let source = r#"
numbers = [1, 2, 3]
scores = {"uno": 10, "dos": 20}
result = numbers[1] + scores["uno"]
flag = result is not None
print(result)
print(flag)
"#;
    assert!(
        compile_text(source, true).is_ok(),
        "lista/dict/None/index no deben romper Python"
    );
}

/// Las funciones Python normales sin anotación de parámetros y retorno deben compilar.
#[test]
fn python_oficial_sin_anotaciones() {
    let source = r#"
def add(a, b):
    return a + b

items = [1, 2, 3]
items.append(4)
score = {"uno": 10, "dos": 20}
print(add(2, 3))
print(items[3])
print(score["uno"])
"#;
    assert!(
        compile_text(source, true).is_ok(),
        "las funciones Python sin tipos deben compilar"
    );
}

/// Las importaciones y la iteración con `for ... in ...` son sintaxis Python real.
#[test]
fn python_imports_y_for_en_lista() {
    let source = r#"
import math
from math import sqrt

values = [1, 2, 3]
acc = 0
for value in values:
    acc += value
print(acc)
print(sqrt(9))
"#;
    assert!(
        compile_text(source, true).is_ok(),
        "import y for-in deben ser compatibles con Python"
    );
}

/// Las comprobaciones de pertenencia y la negación de identidad son Python real.
#[test]
fn python_pertenencia_y_is_not() {
    let source = r#"
values = [1, 2, 3]
inside = 2 in values
missing = 4 not in values
flag = values is not None
print(inside)
print(missing)
print(flag)
"#;
    assert!(
        compile_text(source, true).is_ok(),
        "in, not in e is not deben ser compatibles con Python"
    );
}

/// Ejecuta los ejemplos distribuidos, incluidos recursión, rangos y benchmark.
#[test]
fn ejemplos_nativos() {
    for (source, output) in [
        (
            // El ejemplo hello.st es editable; esta regresión conserva su entrada original.
            "mostrar(\"Hola desde ST: si, funcion, variable\"); mostrar(42);",
            "Hola desde ST: si, funcion, variable\n42\n",
        ),
        (include_str!("../examples/functions.st"), "42\n120\n"),
        (
            include_str!("../examples/control_flow.st"),
            "25\n3\n2\n1\ncorrecto\nTrue\n",
        ),
        (
            // El ejemplo puede editarse en el IDE; la regresión usa una entrada estable.
            "funcion sumar_hasta(n: entero) -> entero { variable total = 0; para i en rango(n) { total += i; } retornar total; } mostrar(sumar_hasta(100000000));",
            "4999999950000000\n",
        ),
    ] {
        native(source, output);
    }
}

/// Comprueba los casos estructurales: bloque vacío, rango de dos límites y seis argumentos.
#[test]
fn estructuras_y_argumentos() {
    native("funcion f(a: entero,b: entero,c: entero,d: entero,e: entero,f: entero) -> entero { retornar a+b+c+d+e+f; } funcion vacia() -> nada {} vacia(); variable s=0; para i en rango(2,5) { s+=i; } si falso {} sino si verdadero { mostrar(s); } sino { mostrar(999); } mostrar(f(1,2,3,4,5,6));", "9\n21\n");
}

/// Comprueba check, emit-ir, emit-asm y run con archivos ST mediante la interfaz real.
#[test]
fn comandos_compilador() {
    let temp = Temp::new();
    let input = temp.0.join("main.st");
    fs::write(&input, "mostrar(42);").unwrap();
    for command in ["check", "emit-ir", "emit-asm", "run"] {
        let result = Command::new(env!("CARGO_BIN_EXE_pyforgec"))
            .current_dir(&temp.0)
            .arg(command)
            .arg(&input)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        if command == "run" {
            assert_eq!(result.stdout, b"42\n");
        }
        if command == "emit-asm" {
            assert!(String::from_utf8_lossy(&result.stdout).contains("_start:"));
        }
        if command == "emit-ir" {
            assert!(String::from_utf8_lossy(&result.stdout).contains("PrintInt"));
        }
    }
    let catalog = Command::new(env!("CARGO_BIN_EXE_pyforgec"))
        .arg("language-json")
        .output()
        .unwrap();
    assert!(catalog.status.success());
    let catalog = String::from_utf8(catalog.stdout).unwrap();
    assert!(catalog.contains("\"publica\""));
    assert!(catalog.contains("\"privada\""));
    assert!(catalog.contains("\"entero\""));
}

/// El gestor acepta entry .st sin cambiar el formato del manifiesto.
#[test]
fn proyecto_st() {
    let temp = Temp::new();
    fs::write(
        temp.0.join("PyForge.toml"),
        "[package]\nname = \"prueba\"\n[build]\nentry = \"main.st\"\n",
    )
    .unwrap();
    fs::write(temp.0.join("main.st"), "mostrar(42);").unwrap();
    for command in ["check", "build", "run", "clean"] {
        let result = Command::new(env!("CARGO_BIN_EXE_pyforge"))
            .current_dir(&temp.0)
            .arg(command)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        if command == "run" {
            assert_eq!(result.stdout, b"42\n");
        }
    }
    assert!(!temp.0.join("target").exists());
    assert!(temp.0.join("main.st").exists());
}

/// new genera un proyecto ST completo y ejecutable sin editar el manifiesto.
#[test]
fn nuevo_proyecto_st_por_defecto() {
    let temp = Temp::new();
    let bin = env!("CARGO_BIN_EXE_pyforge");
    let result = Command::new(bin)
        .current_dir(&temp.0)
        .args(["new", "calculator"])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let project = temp.0.join("calculator");
    assert!(project.join("src/main.st").exists());
    assert!(!project.join("src/main.py").exists());
    let (name, entry) = pyforge::project::project_entry(&project).unwrap();
    assert_eq!(name, "calculator");
    assert_eq!(entry, project.join("src/main.st"));
    for command in ["check", "run"] {
        let result = Command::new(bin)
            .current_dir(&project)
            .arg(command)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        if command == "run" {
            assert_eq!(result.stdout, b"Hola desde ST\n42\n");
        }
    }
    // Un manifiesto anterior sin entry conserva la entrada Python histórica.
    fs::write(
        project.join("PyForge.toml"),
        "[package]\nname = \"calculator\"\n",
    )
    .unwrap();
    assert_eq!(
        pyforge::project::project_entry(&project).unwrap().1,
        project.join("src/main.py")
    );
}

/// Las colecciones crecen, comparten referencia y conservan los valores tras llamadas.
#[test]
fn colecciones_nativas() {
    native(
        include_str!("../examples/collections.st"),
        "22\n40\n4\n21\n3\n",
    );
    native("funcion cambiar(xs: lista) -> lista { agregar(xs, 99); retornar xs; } variable xs: lista=[]; para i en rango(1000) { agregar(xs,i); } variable alias=cambiar(xs); alias[0]=77; mostrar(xs[0]); mostrar(xs[999]); mostrar(xs[-1]); mostrar(longitud(xs)); variable d={\"uno\":1,\"dos\":2,\"uno\":3}; d[\"tres\"]=4; d[\"cuatro\"]=5; d[\"cinco\"]=6; mostrar(d[\"uno\"]); mostrar(d[\"cinco\"]); mostrar(longitud(d));", "77\n999\n99\n1001\n3\n6\n5\n");
}

/// Rechaza tipos de elementos/índices no admitidos durante la compilación.
#[test]
fn tipos_colecciones() {
    for source in [
        "variable x={\"a\":falso,\"b\":1};",
        "variable x=[1]; mostrar(x[\"a\"]);",
        "variable x={}; mostrar(x[0]);",
        "variable x=[1]; x[0]=falso;",
        "agregar(1,2);",
        "longitud(1);",
        "mostrar([1]);",
        "variable x=[1]; x[0]+=1;",
    ] {
        assert!(compile_st_text(source, true).is_err(), "{source}");
    }
}

/// Los errores de acceso se informan en español y terminan con estado 1 sin señales.
#[test]
fn limites_colecciones() {
    for (source, message) in [
        ("variable x=[]; mostrar(x[0]);", "indice fuera de rango"),
        ("mostrar(\"\"[0]);", "indice fuera de rango"),
        ("mostrar(\"ñ\"[-2]);", "indice fuera de rango"),
        (
            "mostrar(reemplazar(\"abc\",\"\",\"x\"));",
            "patron no vacio",
        ),
        ("variable x=[1]; x[-2]=1;", "indice fuera de rango"),
        (
            "variable x={}; mostrar(x[\"ausente\"]);",
            "clave inexistente",
        ),
    ] {
        let temp = Temp::new();
        let input = temp.0.join("main.st");
        let out = temp.0.join("main");
        fs::write(&input, source).unwrap();
        for opt in [true, false] {
            pyforge::driver::build_executable(&input, &out, opt, false).unwrap();
            let result = Command::new(&out).output().unwrap();
            assert_eq!(result.status.code(), Some(1));
            assert!(String::from_utf8_lossy(&result.stderr).contains(message));
        }
    }
}

/// Un módulo común se inicializa una sola vez, incluso en dependencias en diamante.
#[test]
fn importar_modulos() {
    let temp = Temp::new();
    fs::create_dir(temp.0.join("lib")).unwrap();
    fs::write(
        temp.0.join("lib/math.st"),
        "mostrar(7); funcion doble(n: entero)->entero {retornar n*2;}",
    )
    .unwrap();
    fs::write(
        temp.0.join("a.st"),
        "importar \"lib/math.st\"; funcion a()->entero {retornar doble(3);}",
    )
    .unwrap();
    fs::write(
        temp.0.join("b.st"),
        "importar \"lib/../lib/math.st\"; funcion b()->entero {retornar doble(4);}",
    )
    .unwrap();
    let input = temp.0.join("main.st");
    let out = temp.0.join("main");
    fs::write(
        &input,
        "importar \"a.st\"; importar \"b.st\"; mostrar(a()+b());",
    )
    .unwrap();
    for opt in [true, false] {
        pyforge::driver::build_executable(&input, &out, opt, false).unwrap();
        let result = Command::new(&out).output().unwrap();
        assert!(result.status.success());
        assert_eq!(result.stdout, b"7\n14\n");
    }
}

/// Detecta ciclos, dependencias ausentes, funciones duplicadas y errores del módulo.
#[test]
fn errores_importaciones() {
    let temp = Temp::new();
    let main = temp.0.join("main.st");
    fs::write(&main, "importar \"ausente.st\";").unwrap();
    assert!(compile_file(&main, true)
        .err()
        .unwrap()
        .contains("ausente.st"));
    fs::write(&main, "importar \"a.st\";").unwrap();
    fs::write(temp.0.join("a.st"), "importar \"main.st\";").unwrap();
    assert!(compile_file(&main, true)
        .err()
        .unwrap()
        .contains("circular"));
    fs::write(temp.0.join("a.st"), "funcion f()->entero {retornar 1;}").unwrap();
    fs::write(
        &main,
        "importar \"a.st\"; funcion f()->entero {retornar 2;}",
    )
    .unwrap();
    assert!(compile_file(&main, true).is_err());
    fs::write(&main, "importar \"a.st\";").unwrap();
    fs::write(temp.0.join("a.st"), "mostrar(;").unwrap();
    assert!(compile_file(&main, true).err().unwrap().contains("a.st"));
    assert!(compile_st_text("importar \"a.st\";", true).is_err());
}

/// Ambas grafías de la rama alternativa conservan cortocircuito y ejecución nativa.
#[test]
fn ramas_sino_si() {
    native("si falso { mostrar(1); } sino_si verdadero { mostrar(2); } sino { mostrar(3); } si falso { pasar; } sino si verdadero { mostrar(4); }", "2\n4\n");
}

/// Los errores de tokens incluyen código y cursor; los semánticos identifican su etapa.
#[test]
fn diagnosticos_con_contexto() {
    let error = compile_st_text("mostrar(1);\nmostrar(;", true)
        .err()
        .unwrap();
    assert!(error.contains("línea 2"));
    assert!(error.contains("2 | mostrar(;"));
    assert!(error.contains("encontrado: ';'"));
    assert!(error.contains('^'));
    let error = compile_st_text("mostrar(@);", true).err().unwrap();
    assert!(error.contains("1 | mostrar(@);"));
    let error = compile_st_text("mostrar(ausente);", true).err().unwrap();
    assert!(error.contains("análisis semántico"));
    let temp = Temp::new();
    let entry = temp.0.join("entrada.st");
    fs::write(&entry, "importar \"modulo.st\";").unwrap();
    fs::write(temp.0.join("modulo.st"), "mostrar(;").unwrap();
    let error = compile_file(&entry, true).err().unwrap();
    assert!(error.contains("modulo.st"));
    assert!(error.contains("1 | mostrar(;"));
}

/// Los nuevos valores se pasan y retornan sin intérprete; las tuplas permiten lectura.
#[test]
fn tuplas_conjuntos_nulo() {
    native("funcion identidad(t: tupla) -> tupla { retornar t; } funcion vacio() -> nulo { retornar nulo; } variable t: tupla = tupla(10, 20,); variable alias = identidad(t); mostrar(alias[-1]); mostrar(longitud(t)); mostrar(t es alias); variable otro = tupla(10,20); mostrar(t es otro); variable s: conjunto = conjunto(1, 1, -2); s.agregar(3); agregar(s, 3); mostrar(longitud(s)); variable n: nulo = vacio(); mostrar(n); mostrar(n es nulo); variable l = []; l.agregar(42); mostrar(l[0]);", "20\n2\nTrue\nFalse\n3\nnulo\nTrue\n42\n");
    native("variable s = conjunto(); para i en rango(500) { s.agregar(i); s.agregar(i); } mostrar(longitud(s)); mostrar(longitud(tupla()));", "500\n0\n");
}

/// Rechaza mutaciones de tuplas y operaciones sin semántica definida para estos tipos.
#[test]
fn restricciones_nuevos_tipos() {
    for source in [
        "variable t = tupla(1); t[0] = 2;",
        "variable t = tupla(1); agregar(t,2);",
        "variable s = conjunto(1); mostrar(s[0]);",
        "variable t = tupla(verdadero);",
        "variable s = conjunto(nulo);",
        "mostrar(1 es 1);",
        "variable n = nulo; n = 1;",
        "variable s = conjunto(); mostrar(s);",
        "variable t = tupla(1); t.otro(2);",
    ] {
        assert!(compile_st_text(source, true).is_err(), "{source}");
    }
}

/// Las cadenas conservan Unicode, inmutabilidad, parámetros y retorno por referencia.
#[test]
fn cadenas_nativas() {
    native(
        include_str!("../examples/cadenas.st"),
        "Ana\n20\n12.5\nTrue\nHola, Ana\nAna 😀\n5\n25.0\n",
    );
    native("funcion unir(a: cadena,b: cadena)->cadena { retornar a+b; } variable a: cadena = \"\"; variable copia = a; a += \"ñ😀\"; mostrar(copia); mostrar(a); mostrar(longitud(a)); mostrar(unir(a, \"!\") == \"ñ😀!\"); mostrar(\"a\" != \"b\"); mostrar(\"a\" == \"aa\"); mostrar(\"\" == \"\"); variable x = \"x\"; para i en rango(10) { x += \"x\"; } mostrar(longitud(x));", "\nñ😀\n2\nTrue\nTrue\nFalse\nTrue\n11\n");
}

/// Aritmética f64, valores extremos y comparaciones no ordenadas coinciden con/sin opt.
#[test]
fn flotantes_nativos() {
    native("funcion doble(x: flotante)->flotante { retornar x*2.0; } variable x: flotante = 1.25; x += 2.5; mostrar(doble(x)); mostrar(-x); mostrar(1.0/8.0); mostrar(1e20); mostrar(1e-9); mostrar(-0.0); mostrar(2.0>1.0); mostrar(2.0>=2.0); mostrar(1.0<2.0); mostrar(1.0<=1.0); mostrar(-0.0 == 0.0); variable nan = 0.0/0.0; mostrar(nan); mostrar(nan == nan); mostrar(nan != nan); mostrar(nan < 0.0); mostrar(1.0/0.0); mostrar(-1.0/0.0);", "7.5\n-3.75\n0.125\n1.0e+20\n1.0e-9\n-0.0\nTrue\nTrue\nTrue\nTrue\nTrue\nnan\nFalse\nTrue\nFalse\ninf\n-inf\n");
    native("mostrar(5e-324); mostrar(1.7976931348623157e308); mostrar(999999.999999); mostrar(1.0-2.0); mostrar(+2.0);", "4.940656e-324\n1.797693e+308\n999999.999999\n-1.0\n2.0\n");
}

/// Mantiene tipos estables y diagnostica mezclas u operaciones aún no definidas.
#[test]
fn tipos_escalares_invalidos() {
    for source in [
        "variable a: cadena = 1;",
        "variable a: flotante = 1;",
        "mostrar(1.0+1);",
        "mostrar(2.0%1.0);",
        "variable a=\"x\"; a-=\"x\";",
        "mostrar(1e999);",
        "mostrar(1e+);",
        "variable x=1.0; x=verdadero;",
        "mostrar(longitud(1.0));",
    ] {
        assert!(compile_st_text(source, true).is_err(), "{source}");
    }
}

/// Índices y búsquedas cuentan puntos de código, no bytes UTF-8; conversión respeta mostrar.
#[test]
fn operaciones_texto_nativas() {
    native("variable t = \"añ😀z\"; mostrar(t[0]); mostrar(t[1]); mostrar(t[-2]); mostrar(buscar(t,\"😀\")); mostrar(buscar(t,\"x\")); mostrar(buscar(t,\"\")); mostrar(reemplazar(t,\"ñ😀\",\"!\")); mostrar(reemplazar(\"aaaaa\",\"aa\",\"X\")); mostrar(reemplazar(t,\"x\",\"y\")); mostrar(reemplazar(t,\"añ😀z\",\"\")); mostrar(\"edad: \"+20); mostrar(20+\" años\"); mostrar(\"precio: \"+12.5); mostrar(\"activo: \"+verdadero); mostrar(cadena(nulo)); mostrar(interpolar(\"{{nombre}}: {} / {}\",t,42));", "a\nñ\n😀\n2\n-1\n0\na!z\nXXa\nañ😀z\n\nedad: 20\n20 años\nprecio: 12.5\nactivo: True\nnulo\n{nombre}: añ😀z / 42\n");
    native("funcion efecto()->entero { mostrar(99); retornar 2; } mostrar(interpolar(\"{}!\",efecto())); mostrar(cadena(-9223372036854775807-1)); mostrar(cadena(1e20)); mostrar(cadena(1.0/0.0));", "99\n2!\n-9223372036854775808\n1.0e+20\ninf\n");
}

/// Las claves construidas en ejecución se comparan por contenido y conservan sus valores.
#[test]
fn claves_dinamicas() {
    native("variable clave = \"ed\"+\"ad\"; variable d = {clave:20}; mostrar(d[\"edad\"]); d[interpolar(\"{}\",clave)] = 21; mostrar(d[clave]); mostrar(longitud(d)); variable otra = clave; clave = \"nombre\"; d[clave]=42; mostrar(d[otra]); mostrar(longitud(d));", "20\n21\n1\n21\n2\n");
}

/// Rechaza plantillas incompletas y mutación de texto en vez de escribir memoria constante.
#[test]
fn errores_operaciones_texto() {
    for source in [
        "variable s=\"abc\"; s[0]=1;",
        "mostrar(interpolar(\"{}\"));",
        "mostrar(interpolar(\"x\",1));",
        "mostrar(interpolar(\"{nombre}\",1));",
        "mostrar(buscar(\"a\",1));",
        "mostrar(\"x\" + [1]);",
        "mostrar(cadena([1]));",
    ] {
        assert!(compile_st_text(source, true).is_err(), "{source}");
    }
}

/// La inserción de claves textuales mantiene la deduplicación después de crecer.
#[test]
fn claves_y_conversiones_crecimiento() {
    native("variable d = {}; para i en rango(80) { d[\"k\"+i] = i; } mostrar(longitud(d)); mostrar(d[\"k79\"]); d[\"k\"+79] = 100; mostrar(longitud(d)); mostrar(d[\"k79\"]); variable s = \"valor: \"; s += 42; mostrar(s); mostrar(reemplazar(\"ababa\",\"a\",\"XYZ\"));", "80\n79\n80\n100\nvalor: 42\nXYZbXYZbXYZ\n");
}

/// Los errores aritméticos y los límites de recursos terminan con estado 1, sin señales.
#[test]
fn limites_seguridad_nativos() {
    for (source, message) in [
        ("mostrar(9223372036854775807+1);", "desbordamiento"),
        ("mostrar((-9223372036854775807-1)-1);", "desbordamiento"),
        ("mostrar(9223372036854775807*2);", "desbordamiento"),
        (
            "variable x=-9223372036854775807-1; mostrar(-x);",
            "desbordamiento",
        ),
        ("mostrar(1/0);", "por cero"),
        ("mostrar(1%0);", "por cero"),
        (
            "variable x=-9223372036854775807-1; mostrar(x/-1);",
            "desbordamiento",
        ),
        (
            "variable x=-9223372036854775807-1; mostrar(x%-1);",
            "desbordamiento",
        ),
        (
            "funcion f()->entero { retornar f(); } mostrar(f());",
            "limite de pila",
        ),
        ("variable s=\"x\"; mientras verdadero { s=s+s; }", "256 MiB"),
    ] {
        let temp = Temp::new();
        let input = temp.0.join("seguridad.st");
        let out = temp.0.join("seguridad");
        fs::write(&input, source).unwrap();
        for opt in [false, true] {
            pyforge::driver::build_executable(&input, &out, opt, false).unwrap();
            let result = Command::new(&out).output().unwrap();
            assert_eq!(result.status.code(), Some(1), "{source}");
            assert!(
                String::from_utf8_lossy(&result.stderr).contains(message),
                "{source}"
            );
        }
    }
}

/// Un puntero obtenido solo en un bucle no puede leerse después sin inicialización segura.
#[test]
fn flujo_seguro() {
    for source in [
        "mientras falso { variable s=\"x\"; } mostrar(s);",
        "para i en rango(0) { variable l=[1]; } mostrar(l[0]);",
        "funcion f(x: booleano)->cadena { si x { retornar \"x\"; } } mostrar(f(falso));",
        "variable l=[1]; para l en rango(2) { pasar; }",
    ] {
        assert!(compile_st_text(source, true).is_err(), "{source}");
    }
    native("funcion f(x: booleano)->entero { si x { retornar 1; } sino { retornar 2; } } mostrar(10+f(falso)); mostrar((verdadero o falso) y f(verdadero));", "12\nTrue\n");
}

/// Las colecciones conservan su tipo al pasar por funciones, mutar y crecer.
#[test]
fn colecciones_escalares() {
    native("funcion primero(nombres: lista[cadena])->cadena { retornar nombres[0]; } variable nombres: lista[cadena]=[]; nombres.agregar(\"Ana\"); agregar(nombres,\"Luis\"); mostrar(primero(nombres)); nombres[0]=\"Eva\"; mostrar(nombres[0]); variable precios: diccionario[flotante]={}; precios[\"pan\"]=1.5; mostrar(precios[\"pan\"]*2.0); variable flags=[verdadero,falso]; mostrar(flags[1]); variable d={\"ok\":verdadero}; mostrar(d[\"ok\"]); variable n=[nulo]; mostrar(n[0]);", "Ana\nEva\n3.0\nFalse\nTrue\nnulo\n");
    native("variable xs: lista[cadena]=[]; para i en rango(100) { xs.agregar(\"n\"+i); } mostrar(xs[-1]); mostrar(longitud(xs)); variable alias=xs; mostrar(alias es xs);", "n99\n100\nTrue\n");
    for source in [
        "variable x: lista[cadena]=[]; x.agregar(1);",
        "variable x=[\"a\"]; x[0]=2;",
        "variable d={\"a\":1.5}; d[\"b\"]=1;",
    ] {
        assert!(compile_st_text(source, true).is_err(), "{source}");
    }
}

/// Las listas mixtas guardan una etiqueta por posición y conservan orden y mutación.
#[test]
fn listas_heterogeneas_etiquetadas() {
    native(
        r#"
        clase Caja { variable valor: entero; }
        funcion primero(valores: lista[mixto]) -> cadena {
            retornar cadena(valores[0]);
        }
        funcion crear() -> lista[mixto] {
            retornar ["devuelto"; 12];
        }
        variable datos = ["Ana"; 20; 2.5; verdadero; nulo];
        mostrar(tipo(datos));
        mostrar(datos[0]); mostrar(tipo(datos[0]));
        mostrar(datos[1]); mostrar(tipo(datos[1]));
        mostrar(datos[2]); mostrar(tipo(datos[2]));
        mostrar(datos[3]); mostrar(tipo(datos[3]));
        mostrar(datos[4]); mostrar(tipo(datos[4]));
        datos[0] = 99;
        datos.agregar("final");
        variable copiado = datos[1];
        agregar(datos; copiado);
        mostrar(datos[0]); mostrar(datos[-2]); mostrar(datos[-1]);
        intentar { mostrar(entero(datos[-2])); }
        capturar error { mostrar("conversion rechazada"); }
        mostrar(entero(datos[1]) + 2);
        mostrar(flotante(datos[1]));
        mostrar(cadena(datos[3]));
        variable vacia: lista[mixto] = [];
        vacia.agregar("texto"); vacia.agregar(7);
        mostrar(vacia[0]); mostrar(vacia[1]);
        vacia[0] = datos[1]; mostrar(vacia[0]);
        variable enteros = [8; 9];
        mostrar(primero(enteros));
        variable devuelta = crear();
        mostrar(devuelta[0]); mostrar(devuelta[1]);
        variable copias = [datos[1]; datos[3]];
        mostrar(copias[0]); mostrar(copias[1]);
        variable caja = nuevo Caja(5);
        variable compuestos = [caja; [1; 2]];
        mostrar(tipo(compuestos[0])); mostrar(tipo(compuestos[1]));
        variable total = 0;
        para i en rango(70000) { total += entero(datos[1]); }
        mostrar(total);
    "#,
        "lista[mixto]\nAna\ncadena\n20\nentero\n2.5\nflotante\nTrue\nbooleano\nnulo\nnulo\n99\nfinal\n20\nconversion rechazada\n22\n20.0\nTrue\ntexto\n7\n20\n8\ndevuelto\n12\n20\nTrue\nobjeto\nlista\n1400000\n",
    );
    assert!(compile_st_text("variable x: mixto = 1;", true).is_err());
    assert!(compile_st_text("funcion f(x: mixto) -> nada { pasar; }", true).is_err());
    assert!(compile_st_text("funcion f() -> mixto { retornar 1; }", true).is_err());
    assert!(compile_st_text("variable x: diccionario[mixto] = {};", true).is_err());
}

/// Archivos y stdin se prueban solo en directorios temporales, sin tocar datos del usuario.
#[test]
fn entrada_archivos_nativos() {
    use std::{io::Write, process::Stdio};
    let temp = Temp::new();
    let input = temp.0.join("io.st");
    let binary = temp.0.join("io");
    fs::write(&input,"variable a=entrada(); variable b=entrada(); escribir(\"dato.txt\",a+\" / \"+b); mostrar(abrir(\"dato.txt\")); escribir(\"dato.txt\",\"\"); mostrar(longitud(abrir(\"dato.txt\")));").unwrap();
    for opt in [true, false] {
        pyforge::driver::build_executable(&input, &binary, opt, false).unwrap();
        let mut child = Command::new(&binary)
            .current_dir(&temp.0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all("Ana ñ\n😀\n".as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(out.stdout, "Ana ñ / 😀\n0\n".as_bytes());
        assert_eq!(fs::read(temp.0.join("dato.txt")).unwrap(), b"");
    }
}

/// Lecturas ausentes, demasiado grandes o UTF-8 inválido terminan con diagnóstico.
#[test]
fn errores_io_nativos() {
    let temp = Temp::new();
    let input = temp.0.join("main.st");
    let binary = temp.0.join("app");
    fs::write(temp.0.join("invalido"), [0xc0, 0xaf]).unwrap();
    fs::write(temp.0.join("grande"), vec![b'a'; 1048577]).unwrap();
    for (path, message) in [
        ("ausente", "entrada/salida"),
        ("invalido", "UTF-8"),
        ("grande", "1 MiB"),
    ] {
        fs::write(&input, format!("mostrar(abrir(\"{path}\"));")).unwrap();
        for opt in [false, true] {
            pyforge::driver::build_executable(&input, &binary, opt, false).unwrap();
            let result = Command::new(&binary).current_dir(&temp.0).output().unwrap();
            assert_eq!(result.status.code(), Some(1));
            assert!(String::from_utf8_lossy(&result.stderr).contains(message));
        }
    }
}

/// Selección nativa: evaluación única, primer caso, Unicode y retorno en todas las ramas.
#[test]
fn patrones_literales() {
    native("funcion valor()->cadena { mostrar(99); retornar \"ñ\"; } coincidir valor() { caso \"x\" { mostrar(1); } caso \"ñ\" { mostrar(2); } caso \"ñ\" { mostrar(3); } sino { mostrar(4); } } funcion etiqueta(x: entero)->cadena { coincidir x { caso -1 { retornar \"negativo\"; } sino { retornar \"otro\"; } } } mostrar(etiqueta(-1)); coincidir 1.5 { caso 1.5 { mostrar(5); } } coincidir nulo { caso nulo { mostrar(6); } }", "99\n2\nnegativo\n5\n6\n");
    for source in [
        "coincidir 1 { caso \"1\" { pasar; } }",
        "coincidir 1 { caso 1+1 { pasar; } }",
        "coincidir [] { sino { pasar; } }",
        "coincidir 1 { sino {} caso 1 {} }",
        "funcion f(x: entero)->cadena { coincidir x { caso 1 { retornar \"x\"; } } }",
    ] {
        assert!(compile_st_text(source, true).is_err(), "{source}");
    }
}

/// Alias y selección distinguen funciones homónimas, preservando recursión e inicialización única.
#[test]
fn modulos_con_alias() {
    let temp = Temp::new();
    let entry = temp.0.join("main.st");
    let out = temp.0.join("app");
    fs::write(temp.0.join("a.st"),"mostrar(7); funcion doble(x: entero)->entero { retornar x*2; } funcion factorial(n: entero)->entero { si n<=1 { retornar 1; } retornar n*factorial(n-1); }").unwrap();
    fs::write(
        temp.0.join("b.st"),
        "funcion doble(x: entero)->entero { retornar x*3; }",
    )
    .unwrap();
    fs::write(&entry,"importar \"a.st\" como a; importar \"b.st\" como b; desde \"a.st\" importar doble como dos, factorial; mostrar(a.doble(10)); mostrar(b.doble(10)); mostrar(dos(10)); mostrar(factorial(5));").unwrap();
    for opt in [false, true] {
        pyforge::driver::build_executable(&entry, &out, opt, false).unwrap();
        let result = Command::new(&out).output().unwrap();
        assert!(result.status.success());
        assert_eq!(result.stdout, b"7\n20\n30\n20\n120\n");
    }
    fs::write(&entry, "desde \"a.st\" importar ausente;").unwrap();
    assert!(compile_file(&entry, true)
        .err()
        .unwrap()
        .contains("ausente"));
    fs::write(
        &entry,
        "desde \"a.st\" importar doble; desde \"b.st\" importar doble;",
    )
    .unwrap();
    assert!(compile_file(&entry, true)
        .err()
        .unwrap()
        .contains("ambiguos"));
}

/// Los módulos solo publican la API elegida y aíslan sus variables internas.
#[test]
fn visibilidad_de_funciones_y_variables() {
    let temp = Temp::new();
    let module = temp.0.join("config.st");
    let entry = temp.0.join("main.st");
    let output = temp.0.join("app");
    fs::write(
        &module,
        r#"
        publica variable version: cadena = "1.0";
        privada variable secreto: cadena = "interno";
        publica funcion doble(valor: entero) -> entero { retornar valor * 2; }
        privada funcion token() -> cadena { retornar "interno"; }
        "#,
    )
    .unwrap();
    fs::write(
        &entry,
        r#"importar "config.st" como config;
        mostrar(config.version);
        mostrar(config.doble(21));"#,
    )
    .unwrap();
    for opt in [false, true] {
        pyforge::driver::build_executable(&entry, &output, opt, false).unwrap();
        let result = Command::new(&output).output().unwrap();
        assert!(result.status.success());
        assert_eq!(result.stdout, b"1.0\n42\n");
    }

    fs::write(
        &entry,
        "desde \"config.st\" importar token; mostrar(token());",
    )
    .unwrap();
    assert!(compile_file(&entry, true)
        .err()
        .unwrap()
        .contains("símbolo público token no existe"));
    fs::write(
        &entry,
        "importar \"config.st\" como config; mostrar(config.secreto);",
    )
    .unwrap();
    assert!(compile_file(&entry, true).is_err());
    fs::write(
        &entry,
        "importar \"config.st\" como config; config.version = \"2.0\";",
    )
    .unwrap();
    assert!(compile_file(&entry, true)
        .err()
        .unwrap()
        .contains("solo lectura"));
}

/// Lanzar abandona argumentos temporales y marcos recursivos, con y sin optimizador.
#[test]
fn excepciones_entre_funciones() {
    native(
        r#"
        funcion falla(n: entero) -> entero {
            si n == 0 { lanzar "falló ñ 😀"; }
            retornar 1 + falla(n - 1);
        }
        intentar { mostrar(10 + falla(30)); } capturar error { mostrar(error); }
        intentar { mostrar(2); } capturar otro { mostrar("no"); }
        mostrar(3);
    "#,
        "falló ñ 😀\n2\n3\n",
    );
}

/// Finalmente conserva el retorno evaluado, se ejecuta en todas las salidas y puede sustituirlo.
#[test]
fn finalmente_y_salidas() {
    native(
        r#"
        funcion valor() -> entero {
            variable x = 7;
            intentar { retornar x; } finalmente { x = 9; mostrar(x); }
        }
        funcion reemplaza() -> entero {
            intentar { retornar 1; } finalmente { retornar 2; }
        }
        mostrar(valor()); mostrar(reemplaza());
        para i en rango(3) {
            intentar { si i == 0 { continuar; } romper; }
            finalmente { mostrar(i); }
        }
        intentar { mostrar(4); } finalmente { mostrar(5); }
        intentar { lanzar "a"; } capturar e { mostrar(e); } finalmente { mostrar(6); }
        intentar {
            intentar { lanzar "original"; } finalmente {
                intentar { lanzar "interior"; } capturar otro { mostrar(otro); }
                mostrar(7);
            }
        } capturar exterior { mostrar(exterior); }
    "#,
        "9\n7\n2\n0\n1\n4\n5\na\n6\ninterior\n7\noriginal\n",
    );
}

/// El retorno retira registros y un nuevo error del capturador llega al padre tras la limpieza.
#[test]
fn manejadores_anidados_y_retorno() {
    native(
        r#"
        funcion regresa() -> entero {
            intentar { retornar 8; } capturar interno { retornar 9; }
        }
        intentar {
            mostrar(regresa());
            intentar { lanzar "uno"; } capturar e { lanzar e + " dos"; }
            finalmente { mostrar("limpieza"); }
        } capturar e { mostrar(e); }
        intentar { lanzar "nuevo"; } capturar e { mostrar(e); }
        // La limpieza pertenece al bucle exterior, aunque el retorno ocurra en otro.
        funcion bucles() -> entero {
            mientras verdadero {
                intentar { mientras verdadero { retornar 3; } }
                finalmente { romper; }
            }
            retornar 4;
        }
        mostrar(bucles());
    "#,
        "8\nlimpieza\nuno dos\nnuevo\n4\n",
    );
}

/// El diagnóstico distingue mensajes inválidos y locales no inicializados en una excepción.
#[test]
fn excepciones_invalidas() {
    for source in [
        "intentar {}",
        "lanzar 1;",
        "capturar error {}",
        "intentar { variable x = 1; } capturar e { mostrar(x); }",
        "intentar {} capturar e {} mostrar(e);",
        "variable e = 1; intentar {} capturar e {}",
        "intentar {} finalmente { romper; }",
    ] {
        assert!(compile_st_text(source, true).is_err(), "{source}");
    }
    let temp = Temp::new();
    let input = temp.0.join("main.st");
    let binary = temp.0.join("app");
    fs::write(&input, "lanzar \"mensaje ñ\";").unwrap();
    for opt in [false, true] {
        pyforge::driver::build_executable(&input, &binary, opt, false).unwrap();
        let result = Command::new(&binary).output().unwrap();
        assert_eq!(result.status.code(), Some(1));
        assert_eq!(
            String::from_utf8(result.stderr).unwrap(),
            "Error ST: excepción sin capturar: mensaje ñ\n"
        );
    }
}

/// Objetos nominales con campos heterogéneos, métodos, referencias compartidas y excepciones.
#[test]
fn clases_nativas() {
    native(
        r#"
        clase Persona {
            variable nombre: cadena;
            variable edad: entero;
            variable saldo: flotante;
            variable activo: booleano;
            funcion saludo() -> cadena { retornar "Hola " + yo.nombre; }
            funcion cumple() { yo.edad = yo.edad + 1; }
        }
        implementar Persona {
            funcion validar() { si yo.edad < 0 { lanzar "edad inválida"; } }
        }
        funcion presentar(p: Persona) -> cadena { retornar p.saludo(); }
        variable p: Persona = nuevo Persona("Ana", 20, 1.5, verdadero);
        variable alias = p;
        p.cumple();
        mostrar(presentar(p)); mostrar(alias.edad); mostrar(p.saldo); mostrar(p.activo);
        mostrar(p es alias); mostrar(p es nuevo Persona("Ana", 21, 1.5, verdadero));
        p.edad = -1;
        intentar { p.validar(); } capturar e { mostrar(e); } finalmente { mostrar("fin"); }
        estructura Caja { variable persona: Persona; variable datos: lista[cadena]; }
        variable caja = nuevo Caja(p, ["uno", "dos"]);
        mostrar(caja.persona.nombre); mostrar(caja.datos[1]);
    "#,
        "Hola Ana\n21\n1.5\nTrue\nTrue\nFalse\nedad inválida\nfin\nAna\ndos\n",
    );
}

/// Los nombres iguales en módulos distintos no mezclan métodos ni disposiciones de campos.
#[test]
fn clases_en_modulos() {
    let temp = Temp::new();
    let entry = temp.0.join("main.st");
    let binary = temp.0.join("app");
    fs::write(
        temp.0.join("a.st"),
        r#"clase Dato { variable x: cadena; funcion leer()->cadena { retornar yo.x; } }"#,
    )
    .unwrap();
    fs::write(
        temp.0.join("b.st"),
        r#"clase Dato { variable x: entero; funcion leer()->entero { retornar yo.x; } }"#,
    )
    .unwrap();
    fs::write(&entry, r#"importar "a.st" como a; desde "b.st" importar Dato como Numero; variable texto = nuevo a.Dato("ñ"); variable numero = nuevo Numero(42); mostrar(texto.leer()); mostrar(numero.leer());"#).unwrap();
    for opt in [false, true] {
        pyforge::driver::build_executable(&entry, &binary, opt, false).unwrap();
        let result = Command::new(&binary).output().unwrap();
        assert!(result.status.success());
        assert_eq!(result.stdout, "ñ\n42\n".as_bytes());
    }
}

/// La comprobación estática impide leer memoria de objetos con tipos o campos incorrectos.
#[test]
fn clases_invalidas() {
    for source in [
        "clase A { variable x: cadena; } variable a = nuevo A(1);",
        "clase A { variable x: cadena; } variable a = nuevo A(\"x\"); a.x = 1;",
        "clase A {} clase B {} variable a: A = nuevo B();",
        "clase A {} variable a = nuevo A(); mostrar(a.ausente);",
        "clase A {} variable a = nuevo A(); a.ausente();",
        "clase A {} variable a = nuevo A(); mostrar(a);",
        "clase A {} variable a = nuevo A(); mostrar(a[0]);",
        "clase A {} variable a = nuevo A(); mostrar(longitud(a));",
        "clase A {} clase A {}",
        "clase A { variable x: entero; variable x: cadena; }",
        "clase A { funcion f() {} funcion f() {} }",
        "mostrar(yo);",
        "variable x = nuevo 3;",
        "implementar Ausente {}",
    ] {
        assert!(compile_st_text(source, true).is_err(), "{source}");
    }
}

/// El mensaje se evalúa una vez, sale sin salto y la lectura conserva Unicode y EOF.
#[test]
fn entrada_con_mensaje() {
    let temp = Temp::new();
    let entry = temp.0.join("main.st");
    let binary = temp.0.join("app");
    let stdin = temp.0.join("entrada.txt");
    fs::write(&stdin, "Ana ñ 😀\nsegunda\n").unwrap();
    fs::write(
        &entry,
        r#"
        funcion mensaje() -> cadena { mostrar("inicio"); retornar "Ingresa tu nombre: "; }
        variable nombre: cadena = entrada(mensaje());
        mostrar("Hola " + nombre);
        mostrar(entrada());
        mostrar(entrada(""));
    "#,
    )
    .unwrap();
    for opt in [false, true] {
        pyforge::driver::build_executable(&entry, &binary, opt, false).unwrap();
        let result = Command::new(&binary)
            .stdin(fs::File::open(&stdin).unwrap())
            .output()
            .unwrap();
        assert!(result.status.success());
        assert_eq!(
            result.stdout,
            "inicio\nIngresa tu nombre: Hola Ana ñ 😀\nsegunda\n\n".as_bytes()
        );
    }
    for source in [
        "entrada(1);",
        "entrada(verdadero);",
        "entrada(\"a\", \"b\");",
        "entrada(ausente);",
    ] {
        assert!(compile_st_text(source, true).is_err(), "{source}");
    }
}

/// Se pueden encadenar campos y llamadas tras nuevo sin cambiar la precedencia.
#[test]
fn construccion_y_encadenamiento() {
    native(
        r#"
        clase Punto {
            variable x: entero;
            funcion valor() -> entero { retornar yo.x; }
        }
        funcion crear(x: entero) -> Punto { mostrar(1); retornar nuevo Punto(x); }
        mostrar(nuevo Punto(7).x);
        mostrar(nuevo Punto(8).valor());
        crear(9).x = 10;
        mostrar(crear(11).valor());
    "#,
        "7\n8\n1\n1\n11\n",
    );
}

/// Ambos formatos son espacios léxicos; no ejecutan contenido ni alteran división o cadenas.
#[test]
fn comentarios_multilinea_nativos() {
    native(
        include_str!("../examples/comentarios.st"),
        "Hola, Ana\n5\n/* esto es texto */\n",
    );
    native(
        r####"
        /**/ """"""
        /* """ no cambia el tipo de comentario # // */
        """ /* tampoco cambia el tipo de comentario */ """
        funcion suma(/* parámetros */ a: entero, b: entero) -> entero {
            retornar a /* separación */ + b;
        }
        mostrar(suma(2, 3));
        mostrar('"""');
        mostrar("comillas: \"\"\"");
        mostrar(12/3); // /* no abre un comentario de bloque
        # """ tampoco abre uno
    "####,
        "5\n\"\"\"\ncomillas: \"\"\"\n4\n",
    );
}

/// Las posiciones avanzan por Unicode y saltos de línea; los errores apuntan a la apertura.
#[test]
fn comentarios_multilinea_diagnosticos() {
    let tokens = tokenize("/* ñ\n😀 */\n\"\"\"uno\ndos\"\"\"\nmostrar(1);").unwrap();
    assert_eq!(tokens[0].line, 5);
    assert_eq!(tokens[0].column, 1);
    for source in ["\n  /* falta cerrar", "\n  \"\"\"falta cerrar", "/*/"] {
        let error = tokenize(source).unwrap_err();
        assert!(
            error.contains("comentario multilínea sin cerrar"),
            "{error}"
        );
    }
    // Los comentarios separan tokens: no unen dos partes de un identificador.
    assert!(compile_st_text("var/* texto */iable x = 1;", true).is_err());
    // Las comillas triples no producen un valor de tipo cadena.
    assert!(compile_st_text("variable x = \"\"\"texto\"\"\";", true).is_err());
}

/// Los cinco módulos incrustados se enlazan como ELF y funcionan con ambas optimizaciones.
#[test]
fn biblioteca_estandar_nativa() {
    native(include_str!("../examples/biblioteca_estandar.st"),
        "9.0\n0.125\nTrue\nTrue\nTrue\nBiblioteca estándar ST\nTrue\nAna\n{\"activo\":false}\n\"Hola, mundo\\u000a\"\nñ 🚀\nFalse\n");
}

/// Alias, selección y archivos math.st locales conservan identidades distintas.
#[test]
fn biblioteca_importaciones() {
    let temp = Temp::new();
    fs::write(
        temp.0.join("math.st"),
        "funcion doble(x: entero) -> entero { retornar x * 2; }",
    )
    .unwrap();
    let file = temp.0.join("main.st");
    fs::write(&file, r#"importar "math" como m; desde "math" importar raiz como r;
        importar "math.st" como local; mostrar(m.raiz(4.0)); mostrar(r(9.0)); mostrar(local.doble(5));"#).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_pyforgec"))
        .arg("run")
        .arg(&file)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout, b"2.0\n3.0\n10\n");
    for source in [
        "importar \"math\"; mostrar(math.raiz(4));",
        "importar \"math\"; mostrar(math.inexistente());",
        "desde \"math\" importar aux_raiz; mostrar(aux_raiz(4.0));",
        "mostrar(math.raiz(4.0));",
        "importar \"time\"; time.dormir(1.0);",
        "importar \"json\"; mostrar(json.codificar(2));",
    ] {
        fs::write(&file, source).unwrap();
        assert!(compile_file(&file, true).is_err(), "{source}");
    }
}

/// Los errores de dominio se capturan en ST; exponentes extremos no desbordan al negarse.
#[test]
fn biblioteca_math_time() {
    native(
        r#"importar "math"; importar "time";
        mostrar(math.absoluto(-2.5)); mostrar(math.absoluto_entero(-7));
        mostrar(math.minimo(2.0, 3.0)); mostrar(math.maximo(2.0, 3.0));
        mostrar(math.potencia(2.0, 10)); mostrar(math.potencia(0.0, 0));
        mostrar(math.potencia(1.0, -9223372036854775807 - 1));
        mostrar(math.raiz(0.0)); mostrar(math.pi() > 3.14); mostrar(math.euler() > 2.71);
        intentar { mostrar(math.raiz(-1.0)); } capturar error { mostrar("dominio"); }
        intentar { mostrar(math.potencia(0.0, -1)); } capturar error { mostrar("exponente"); }
        intentar { time.dormir(-1); } capturar error { mostrar("duración"); }
        variable inicio: entero = time.monotono(); time.dormir(20);
        mostrar(time.monotono() - inicio >= 19); time.dormir(0);
    "#,
        "2.5\n7\n2.0\n3.0\n1024.0\n1.0\n1.0\n0.0\nTrue\nTrue\ndominio\nexponente\nduración\nTrue\n",
    );
}

/// IO y OS operan sobre un directorio de prueba aislado y notifican errores capturables.
#[test]
fn biblioteca_archivos() {
    let temp = Temp::new();
    let dir = temp.0.join("datos").display().to_string();
    native(
        &format!(
            r#"importar "os"; importar "io";
        mostrar(os.existe("{dir}/texto.txt"));
        si no os.existe("{dir}") {{ os.crear_directorio("{dir}"); }}
        io.escribir_texto("{dir}/texto.txt", "Hola ñ 🚀\n");
        mostrar(io.leer_texto("{dir}/texto.txt")); mostrar(os.existe("{dir}/texto.txt"));
        intentar {{ os.crear_directorio("{dir}"); }} capturar error {{ mostrar("ya existe"); }}
        os.borrar_archivo("{dir}/texto.txt"); mostrar(os.existe("{dir}/texto.txt"));
        intentar {{ os.borrar_archivo("{dir}/texto.txt"); }} capturar error {{ mostrar("ausente"); }}
    "#
        ),
        "False\nHola ñ 🚀\n\nTrue\nya existe\nFalse\nausente\n",
    );
}

/// Contrasta la gramática JSON: espacios, números, escapes, anidación y entradas truncadas.
#[test]
fn biblioteca_json_validacion() {
    let good = [
        "null",
        "true",
        "false",
        "0",
        "-0",
        "-1.25e+10",
        "1E-400",
        "99999999999999999999999",
        "[]",
        "{}",
        " [1, {\"a\":[false,null]}] \r\n",
        "\"\\u0000\\ud83d\\ude80\"",
        "\"á\\/\\b\\f\\n\\r\\t\"",
        "{\"a\":1,\"a\":2}",
    ];
    let bad = [
        "",
        " ",
        "01",
        "-01",
        "+1",
        ".5",
        "1.",
        "1e",
        "1e+",
        "NaN",
        "Infinity",
        "True",
        "nul",
        "true false",
        "[1,]",
        "[,1]",
        "{\"a\":}",
        "{a:1}",
        "{\"a\" 1}",
        "{\"a\":1,}",
        "[",
        "{",
        "\"",
        "\"\\x\"",
        "\"\\u12\"",
        "\"\\ud800\"",
        "\"\\udc00\"",
        "\"\\ud800\\u0041\"",
        "\"x\ny\"",
        "\"x\ty\"",
    ];
    let mut source = String::from("importar \"json\";\n");
    for text in good.iter().chain(bad.iter()) {
        // Debug de Rust escapa comillas/LF/CR/tabuladores con formas también válidas en ST.
        source.push_str(&format!("mostrar(json.validar({text:?}));\n"));
    }
    native(
        &source,
        &("True\n".repeat(good.len()) + &"False\n".repeat(bad.len())),
    );
}

/// El decodificador conserva Unicode y controles y permite recorrer JSON heterogéneo anidado.
#[test]
fn biblioteca_json_valores() {
    native(r#"importar "json";
        variable texto: cadena = "á\n\t\\\"🚀";
        mostrar(json.decodificar(json.codificar(texto)) == texto);
        mostrar(json.decodificar("\"\\u007f\\u0080\\u07ff\\u0800\\uffff\\udbff\\udfff\"") == "߿ࠀ￿􏿿");
        mostrar(json.codificar(json.decodificar("\"\\u0000\\b\\f\"")));
        variable objeto: cadena = "{\"\\u0061\":[null,{\"b\":true}],\"a\":2}";
        mostrar(json.elemento(json.obtener(objeto, "a"), 0));
        mostrar(json.obtener(json.elemento(json.obtener(objeto, "a"), 1), "b"));
        mostrar(json.de_entero(-9223372036854775807 - 1));
        mostrar(json.de_booleano(verdadero)); mostrar(json.valor_nulo());
        intentar { mostrar(json.obtener("{}", "x")); } capturar error { mostrar("clave"); }
        intentar { mostrar(json.elemento("[]", 0)); } capturar error { mostrar("índice"); }
        intentar { mostrar(json.elemento("[1]", -1)); } capturar error { mostrar("negativo"); }
        intentar { mostrar(json.decodificar("123")); } capturar error { mostrar("tipo"); }
    "#, "True\nTrue\n\"\\u0000\\u0008\\u000c\"\nnull\ntrue\n-9223372036854775808\ntrue\nnull\nclave\níndice\nnegativo\ntipo\n");
}

/// Constructores JSON y conversiones explícitas comprueban tipos y extremos i64.
#[test]
fn biblioteca_json_construccion() {
    native(r#"importar "json";
        variable valores: lista[cadena] = [json.codificar("Ana"), json.de_entero(20), json.de_booleano(verdadero)];
        variable claves: lista[cadena] = ["nombre", "edad", "activo"];
        variable documento: cadena = json.objeto(claves, valores);
        mostrar(json.decodificar(json.obtener(documento, "nombre")));
        mostrar(json.a_entero(json.obtener(documento, "edad")));
        mostrar(json.a_booleano(json.obtener(documento, "activo")));
        mostrar(json.tipo(json.arreglo(valores)));
        mostrar(json.a_entero(" -9223372036854775808 "));
        mostrar(json.a_entero("9223372036854775807")); mostrar(json.a_entero("-0"));
        intentar { mostrar(json.a_entero("9223372036854775808")); } capturar error { mostrar("máximo"); }
        intentar { mostrar(json.a_entero("-9223372036854775809")); } capturar error { mostrar("mínimo"); }
        intentar { mostrar(json.a_entero("1.0")); } capturar error { mostrar("fracción"); }
        intentar { mostrar(json.a_entero("1e0")); } capturar error { mostrar("exponente"); }
        intentar { mostrar(json.a_booleano("0")); } capturar error { mostrar("tipo"); }
    "#, "Ana\n20\nTrue\narreglo\n-9223372036854775808\n9223372036854775807\n0\nmáximo\nmínimo\nfracción\nexponente\ntipo\n");
}

/// Los límites se rechazan antes de desbordar pila o convertir Unicode inválido.
#[test]
fn biblioteca_json_limites() {
    let valid = "[".repeat(64) + "0" + &"]".repeat(64);
    let invalid = "[".repeat(65) + "0" + &"]".repeat(65);
    let max = format!("\"{}\"", "a".repeat(4094));
    let large = format!("\"{}\"", "a".repeat(4095));
    native(&format!("importar \"json\"; mostrar(json.validar({valid:?})); mostrar(json.validar({invalid:?})); mostrar(json.validar({max:?})); mostrar(json.validar({large:?}));"), "True\nFalse\nTrue\nFalse\n");
}

/// tipo evalúa una vez y describe escalares, colecciones y objetos sin leer su representación.
#[test]
fn consulta_tipo_nativa() {
    native(r#"
        clase Persona { variable nombre: cadena; }
        funcion obtener() -> entero { mostrar("evaluada"); retornar 7; }
        variable nombre: cadena = "Ana";
        mostrar(tipo(nombre)); mostrar(tipo(20)); mostrar(tipo(1.5));
        mostrar(tipo(verdadero)); mostrar(tipo(nulo)); mostrar(tipo([]));
        mostrar(tipo(["a"])); mostrar(tipo({"edad": 20})); mostrar(tipo({"nombre": "Ana"}));
        mostrar(tipo(tupla(1,2))); mostrar(tipo(conjunto(1,2)));
        mostrar(tipo(nuevo Persona("Ana"))); mostrar(tipo(obtener()));
        mostrar(tipo(tipo(1))); mostrar(tipo(1 + 2) == "entero");
    "#, "cadena\nentero\nflotante\nbooleano\nnulo\nlista[entero]\nlista[cadena]\ndiccionario[entero]\ndiccionario[cadena]\ntupla\nconjunto\nobjeto\nevaluada\nentero\ncadena\nTrue\n");
    for source in [
        "tipo();",
        "tipo(1,2);",
        "tipo(ausente);",
        "funcion f() -> nada {} tipo(f());",
        "tipo(mostrar(1));",
    ] {
        assert!(compile_st_text(source, true).is_err(), "{source}");
    }
}

/// La consulta no reserva nombres existentes ni cambia la función json.tipo del módulo estándar.
#[test]
fn consulta_tipo_compatibilidad() {
    native(
        "funcion tipo(x: entero) -> entero { retornar x + 1; } mostrar(tipo(4));",
        "5\n",
    );
    native(
        r#"importar "json"; mostrar(tipo(1)); mostrar(json.tipo("1"));"#,
        "entero\nnumero\n",
    );
    native(
        r#"desde "json" importar tipo; mostrar(tipo("[]"));"#,
        "arreglo\n",
    );
    native(
        "variable tipo: entero = 9; mostrar(tipo); mostrar(tipo(tipo));",
        "9\nentero\n",
    );
}

/// Las conversiones son nativas, explícitas y compatibles con inferencia sin anotaciones.
#[test]
fn conversiones_escalares() {
    native(r#"
        variable texto = "  +42\t"; variable numero = entero(texto);
        mostrar(numero); mostrar(tipo(numero)); mostrar(tipo(texto));
        mostrar(entero("-9223372036854775808")); mostrar(entero("9223372036854775807"));
        mostrar(entero(-3.9)); mostrar(entero(verdadero)); mostrar(flotante(2));
        mostrar(flotante(falso)); mostrar(flotante(" -.125e2 "));
        mostrar(flotante("1.")); mostrar(flotante("000.0005") == 0.0005);
        mostrar(booleano("")); mostrar(booleano("falso")); mostrar(booleano(nulo));
        mostrar(booleano(-0.0)); mostrar(booleano(0)); mostrar(booleano([0]));
        mostrar(booleano([])); mostrar(booleano({"x":0})); mostrar(booleano({}));
        mostrar(cadena(entero("20")) == "20");
    "#, "42\nentero\ncadena\n-9223372036854775808\n9223372036854775807\n-3\n1\n2.0\n0.0\n-12.5\n1.0\nTrue\nFalse\nTrue\nFalse\nFalse\nFalse\nTrue\nFalse\nTrue\nFalse\nTrue\n");
}

/// Los errores de conversión se capturan sin ejecutar instrucciones de desbordamiento inválidas.
#[test]
fn conversiones_errores_y_limites() {
    for expression in [
        "entero(\"\")",
        "entero(\" \t\")",
        "entero(\"+\")",
        "entero(\"1.0\")",
        "entero(\"1e2\")",
        "entero(\"9223372036854775808\")",
        "entero(\"-9223372036854775809\")",
        "entero(9223372036854775808.0)",
        "flotante(\".\")",
        "flotante(\"1e\")",
        "flotante(\"1e+\")",
        "flotante(\"1.2.3\")",
        "flotante(\"nan\")",
        "flotante(\"1e400\")",
        "flotante(\"1e10001\")",
        "flotante(\"1 x\")",
    ] {
        native(&format!("intentar {{ mostrar({expression}); }} capturar error {{ mostrar(\"capturado\"); }} mostrar(7);"), "capturado\n7\n");
    }
    native(
        "mostrar(entero(-9223372036854775808.0)); mostrar(flotante(\"1e-400\"));",
        "-9223372036854775808\n0.0\n",
    );
    for source in [
        "entero();",
        "flotante(1,2);",
        "entero([]);",
        "flotante(nulo);",
        "funcion f() -> nada {} booleano(f());",
        "variable edad = \"20\"; edad = entero(edad);",
    ] {
        assert!(compile_st_text(source, true).is_err(), "{source}");
    }
}

/// La opción web conserva new normal y crea vistas ST compilables y recursos separados.
#[test]
fn proyecto_web_con_vistas_nativas() {
    let temp = Temp::new();
    let result = Command::new(env!("CARGO_BIN_EXE_pyforge"))
        .args(["new", "sitio", "--web"])
        .current_dir(&temp.0)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let root = temp.0.join("sitio");
    for file in [
        "src/main.st",
        "src/vistas.st",
        "web/templates/st.html",
        "web/static/site.css",
        "web/static/site.js",
        "web/requirements.txt",
        "web/comunidad/migrations/0001_initial.py",
    ] {
        assert!(root.join(file).is_file(), "falta {file}");
    }
    assert!(compile_file(&root.join("src/main.st"), true).is_ok());
    let again = Command::new(env!("CARGO_BIN_EXE_pyforge"))
        .args(["new", "sitio", "--web"])
        .current_dir(&temp.0)
        .output()
        .unwrap();
    assert!(!again.status.success());
    assert!(!root.join("web/data/sitio.sqlite3").exists());
}

#[test]
fn proyecto_web_sin_login_y_con_culebrita_st() {
    let temp = Temp::new();
    let result = Command::new(env!("CARGO_BIN_EXE_pyforge"))
        .args(["new", "sitio", "--web"])
        .current_dir(&temp.0)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let root = temp.0.join("sitio");
    let urls = std::fs::read_to_string(root.join("web/urls.py")).unwrap();
    let views = std::fs::read_to_string(root.join("web/comunidad/views.py")).unwrap();
    let st_template = std::fs::read_to_string(root.join("web/templates/st.html")).unwrap();
    let snake_template = std::fs::read_to_string(root.join("web/templates/snake.html")).unwrap();
    assert!(
        !urls.contains("cuenta/entrar"),
        "no debe forzar login en la ruta principal"
    );
    assert!(
        !views.contains("login_required"),
        "no debe haber restricciones de login"
    );
    assert!(
        st_template.contains("Cambiar desde ST"),
        "la vista ST debe mostrar el botón solicitado"
    );
    assert!(
        snake_template.contains("snake.js"),
        "el juego debe usar un script externo compatible con CSP"
    );
    let js = std::fs::read_to_string(root.join("web/static/snake.js")).unwrap();
    assert!(js.contains("/api/snake/"));
}

/// Los puntos y coma no consumen los terminadores externos ni rompen las llamadas anidadas.
#[test]
fn argumentos_con_punto_y_coma() {
    native(
        r#"
        funcion sumar(a: entero; b: entero) -> entero { retornar a + b; }
        variable total = sumar(sumar(1; 2); 3);
        mostrar(total);
        variable datos = [4; 5];
        agregar(datos; total);
        mostrar(longitud(datos));
        mostrar(sumar(7, 8));
    "#,
        "6\n3\n15\n",
    );
}

/// Las llamadas incorporadas validan tipos antes de acceder a referencias o reinterpretar bits.
#[test]
fn builtins_seguros_y_motor_2d() {
    for source in [
        "mostrar(len(1));",
        "mostrar(sqrt(\"x\"));",
        "mostrar(abs([]));",
    ] {
        assert!(compile_st_text(source, true).is_err(), "{source}");
    }
    native(
        "mostrar(abs(-7)); mostrar(sqrt(16)); mostrar(len(\"ñ😀\"));",
        "7\n4.0\n2\n",
    );
    native(
        r#"importar "juego";
        variable cuerpo = [208;207;206];
        variable paso = juego.avanzar_serpiente(cuerpo; 0; 209);
        mostrar(paso[0]); mostrar(longitud(paso));
        mostrar(longitud(juego.avanzar_serpiente([19;18;17];0;100)));
        mostrar(longitud(juego.avanzar_serpiente(cuerpo;2;212)));
        mostrar(juego.comida_libre([0;1;2];0));
    "#,
        "209\n4\n0\n0\n3\n",
    );
}

/// Comprueba ejecutables Python reales, no solo el resultado de check.
#[test]
fn python_regresiones_ejecutables() {
    let temp = Temp::new();
    let source = temp.0.join("caso.py");
    let binary = temp.0.join("caso");
    for (code, expected) in [
        (
            "v=[1,2,3]\nprint(2 in v)\nprint(4 not in v)\n",
            "True\nTrue\n",
        ),
        (
            "print(abs(-7))\nprint(sqrt(16))\nprint(abs(-1.5))\n",
            "7\n4.0\n1.5\n",
        ),
        (
            "def unir(x: str) -> str:\n    return x + x\nprint(unir(\"hola\"))\n",
            "holahola\n",
        ),
        (
            "def doble(x: float) -> float:\n    return x + x\nprint(doble(1.5))\n",
            "3.0\n",
        ),
    ] {
        fs::write(&source, code).unwrap();
        for optimize in [true, false] {
            pyforge::driver::build_executable(&source, &binary, optimize, false).unwrap();
            let result = Command::new(&binary).output().unwrap();
            assert!(result.status.success());
            assert_eq!(String::from_utf8(result.stdout).unwrap(), expected);
        }
    }
    for code in [
        "print(len(1))",
        "def f(x):\n    return x+x\nprint(f(\"texto\"))",
        "else:\n    print(1)",
    ] {
        assert!(compile_text(code, true).is_err(), "{code}");
    }
}

/// Matrices y diccionarios conservan referencias, tipos y comprobaciones de índice.
#[test]
fn colecciones_anidadas() {
    native(
        r#"
        funcion primero(xs: lista[lista[cadena]]) -> cadena { retornar xs[0][0]; }
        variable matriz = [[1; 2]; [3; 4]];
        matriz[1][0] = 9;
        mostrar(matriz[1][0]);
        variable datos: diccionario[lista[cadena]] = {};
        datos["nombres"] = ["Ana"; "Luis"];
        datos["nombres"].agregar("Eva");
        mostrar(datos["nombres"][-1]);
        mostrar(primero([["hola"]]));
        mostrar(tipo(datos));
        variable filas: lista[lista[entero]] = [];
        filas.agregar([7]);
        mostrar(filas[0][0]);
    "#,
        "9\nEva\nhola\ndiccionario[lista[cadena]]\n7\n",
    );
    for source in [
        "variable x=[[1]]; x[0]=2;",
        "variable x=[[1]]; x[0][0]=\"texto\";",
        "variable x=[[1]]; x.agregar(x);",
    ] {
        assert!(compile_st_text(source, true).is_err(), "{source}");
    }
    let deep = format!("variable x={}1{};", "[".repeat(24), "]".repeat(24));
    assert!(compile_st_text(&deep, true).is_err());
}

/// Las bibliotecas locales compilan sin red y no dependen del directorio de ejecución.
#[test]
fn bibliotecas_locales() {
    let root = Temp::new();
    fs::write(root.0.join("PyForge.toml"), "[package]\nname=\"prueba\"\n").unwrap();
    let library = root.0.join("bibliotecas/calculo/src");
    fs::create_dir_all(&library).unwrap();
    fs::write(
        library.join("lib.st"),
        "funcion doble(x: entero)->entero { retornar x*2; }",
    )
    .unwrap();
    let entry = root.0.join("main.st");
    fs::write(
        &entry,
        "importar \"@calculo\" como c; mostrar(c.doble(21));",
    )
    .unwrap();
    let output = root.0.join("programa");
    for optimize in [true, false] {
        pyforge::driver::build_executable(&entry, &output, optimize, false).unwrap();
        let result = Command::new(&output).output().unwrap();
        assert!(result.status.success());
        assert_eq!(result.stdout, b"42\n");
    }
    for name in ["@ausente", "@../calculo", "@"] {
        fs::write(&entry, format!("importar \"{name}\" como c;")).unwrap();
        assert!(compile_file(&entry, true).is_err());
    }
}

/// La ABI Rust trata buffers como recursos comprobados y reutiliza ranuras sin revivir alias.
#[test]
fn runtime_nativo_buffers() {
    native(
        r#"
        importar "nativo";
        mostrar(nativo.version());
        variable b = nativo.desde_texto("Hola ñ");
        mostrar(nativo.tamano(b));
        mostrar(nativo.texto(b));
        nativo.poner(b; 0; 104);
        mostrar(nativo.texto(b));
        variable errores = 0;
        intentar { nativo.obtener(b; -1); } capturar e { errores += 1; }
        intentar { nativo.obtener(b; 7); } capturar e { errores += 1; }
        intentar { nativo.poner(b; 0; 256); } capturar e { errores += 1; }
        nativo.poner(b; 0; 255);
        intentar { nativo.texto(b); } capturar e { errores += 1; }
        nativo.cerrar(b);
        intentar { nativo.cerrar(b); } capturar e { errores += 1; }
        variable nuevo_buffer = nativo.buffer(0);
        intentar { nativo.tamano(b); } capturar e { errores += 1; }
        mostrar(nativo.tamano(nuevo_buffer));
        nativo.cerrar(nuevo_buffer);
        intentar { nativo.buffer(65537); } capturar e { errores += 1; }
        intentar { nativo.buffer(-1); } capturar e { errores += 1; }
        mostrar(errores);
        para i en rango(2000) {
            variable temporal = nativo.buffer(16);
            nativo.poner(temporal; 0; 42);
            nativo.cerrar(temporal);
        }
        variable ids: lista = [];
        para i en rango(32) { ids.agregar(nativo.buffer(1)); }
        intentar { nativo.buffer(1); } capturar e { mostrar("limite"); }
        para i en rango(32) { nativo.cerrar(ids[i]); }
        mostrar("recursos liberados");
    "#,
        "1\n7\nHola ñ\nhola ñ\n0\n8\nlimite\nrecursos liberados\n",
    );
}

/// Prueba binarios y errores de archivos sin Python disponible ni herramientas en PATH.
#[test]
fn runtime_nativo_archivos() {
    let temp = Temp::new();
    let input = temp.0.join("programa.st");
    let output = temp.0.join("programa");
    fs::write(
        &input,
        r#"
        importar "nativo";
        variable b = nativo.buffer(3);
        nativo.poner(b; 0; 0);
        nativo.poner(b; 1; 255);
        nativo.poner(b; 2; 65);
        variable f = nativo.abrir_archivo("dato.bin"; 1);
        intentar { mostrar(nativo.escribir_archivo(f; b)); }
        finalmente { nativo.cerrar(f); nativo.cerrar(b); }
        intentar { nativo.abrir_archivo("dato.bin"; 1); }
        capturar e { mostrar("no sobrescribe"); }
        f = nativo.abrir_archivo("dato.bin"; 0);
        b = nativo.leer(f; 65536);
        mostrar(nativo.tamano(b));
        mostrar(nativo.obtener(b; 1));
        intentar { nativo.escribir_archivo(f; b); }
        capturar e { mostrar("solo lectura"); }
        nativo.cerrar(b);
        b = nativo.leer(f; 1);
        mostrar(nativo.tamano(b));
        nativo.cerrar(b);
        nativo.cerrar(f);
        intentar { nativo.leer(f; 1); } capturar e { mostrar("cerrado"); }
        intentar { nativo.abrir_archivo("inexistente"; 0); }
        capturar e { mostrar("ausente"); }
        f = nativo.abrir_archivo("dato.bin"; 2);
        nativo.cerrar(f);
    "#,
    )
    .unwrap();
    for opt in [true, false] {
        let mut build = Command::new(env!("CARGO_BIN_EXE_pyforgec"));
        build
            .args(["build"])
            .arg(&input)
            .arg("-o")
            .arg(&output)
            .env("PYFORGE_PYTHON", temp.0.join("python-no-existe"));
        if !opt {
            build.arg("--no-opt");
        }
        let result = build.output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let result = Command::new(&output)
            .current_dir(&temp.0)
            .env_clear()
            .env("PATH", "")
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{:?}: {}",
            result.status,
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&result.stdout),
            "3\nno sobrescribe\n3\n255\nsolo lectura\n0\ncerrado\nausente\n"
        );
        assert_eq!(fs::metadata(temp.0.join("dato.bin")).unwrap().len(), 0);
        fs::remove_file(temp.0.join("dato.bin")).unwrap();
        let headers = Command::new("readelf")
            .arg("-l")
            .arg(&output)
            .output()
            .unwrap();
        assert!(headers.status.success());
        assert!(!String::from_utf8_lossy(&headers.stdout).contains("INTERP"));
    }
}

/// Usar cierra al retornar, lanzar, romper y continuar, incluso con ámbitos anidados.
#[test]
fn recursos_propiedad_y_limpieza() {
    let root = Temp::new();
    let entry = root.0.join("main.st");
    let output = root.0.join("programa");
    fs::write(
        &entry,
        r#"
        importar "recursos";
        funcion fabrica() -> buffer_nativo { retornar recursos.desde_texto("Hola"); }
        funcion ver(datos: buffer_nativo) -> cadena { retornar recursos.texto(datos); }
        funcion devolver() -> entero {
            usar b = fabrica() { retornar recursos.tamano(b); }
        }
        funcion falla(datos: buffer_nativo) -> nada { lanzar "prueba"; }
        // `usar` conserva su uso anterior como nombre de función.
        funcion usar(valor: entero) -> entero { retornar valor + 1; }
        mostrar(usar(4));
        para i en rango(100) {
            mostrar(devolver());
            intentar {
                usar externo = fabrica() {
                    usar interno = recursos.buffer(4) { falla(interno); }
                }
            } capturar e { pasar; }
            usar datos = fabrica() {
                si i == 0 { mostrar(tipo(datos)); mostrar(ver(datos)); }
                mientras verdadero {
                    usar temporal = recursos.buffer(1) { romper; }
                }
                continuar;
            }
        }
        mostrar("fin");
    "#,
    )
    .unwrap();
    for opt in [true, false] {
        pyforge::driver::build_executable(&entry, &output, opt, false).unwrap();
        let result = Command::new(&output).output().unwrap();
        assert!(
            result.status.success(),
            "{:?} {}",
            result.status,
            String::from_utf8_lossy(&result.stderr)
        );
        let expected = format!("5\n4\nbuffer_nativo\nHola\n{}fin\n", "4\n".repeat(99));
        assert_eq!(String::from_utf8_lossy(&result.stdout), expected);
    }
}

/// El módulo de teclado compila en ambos modos y distingue una tubería de un TTY.
/// Las pruebas normales capturan stdout, por lo que stdin no debe tratarse como terminal.
#[test]
fn teclado_terminal_nativo() {
    native(
        r#"
        importar "teclado";
        mostrar(teclado.es_terminal());
        intentar { teclado.leer(0); }
        capturar e { mostrar("sin terminal"); }
        mostrar(teclado.arriba());
        mostrar(teclado.abajo());
        mostrar(teclado.derecha());
        mostrar(teclado.izquierda());
        "#,
        "False\nsin terminal\n1001\n1002\n1003\n1004\n",
    );
}

/// El límite de espera se valida en ST antes de modificar la configuración del terminal.
#[test]
fn teclado_rechaza_esperas_invalidas() {
    native(
        r#"
        importar "teclado";
        variable errores = 0;
        intentar { teclado.leer(-1); } capturar e { errores += 1; }
        intentar { teclado.leer(1001); } capturar e { errores += 1; }
        mostrar(errores);
        "#,
        "2\n",
    );
}

/// El núcleo 3D debe ejecutar la misma escena, física y matrices con ambas optimizaciones.
#[test]
fn motor_3d_nucleo_nativo() {
    native(
        r#"
        importar "motor3d";
        variable motor = motor3d.crear_motor("prueba"; 800; 600);
        mostrar(motor.escena.malla_principal.cantidad_vertices());
        mostrar(motor.escena.malla_principal.cantidad_triangulos());
        motor.iniciar();
        motor.actualizar(0.25);
        mostrar(motor.cuadro);
        variable matriz = motor3d.matriz_traslacion(motor3d.vector(2.0; 3.0; 4.0));
        variable punto = matriz.transformar_punto(motor3d.vector(1.0; 1.0; 1.0));
        mostrar(punto.x); mostrar(punto.componente_y); mostrar(punto.z);
        variable a = nuevo motor3d.Caja3D(motor3d.vector(0.0; 0.0; 0.0); motor3d.vector(2.0; 2.0; 2.0));
        variable b = nuevo motor3d.Caja3D(motor3d.vector(1.0; 1.0; 1.0); motor3d.vector(3.0; 3.0; 3.0));
        mostrar(a.intersecta(b));
        variable recursos = motor3d.crear_gestor_recursos(4096);
        variable id_recurso = recursos.registrar("triangulo"; "malla"; 512);
        mostrar(recursos.esta_cargado(id_recurso));
        mostrar(recursos.usado);
        recursos.liberar(id_recurso);
        mostrar(recursos.disponible());
        "#,
        "8\n12\n1\n3.0\n4.0\n5.0\nTrue\nTrue\n512\n4096\n",
    );
}

/// El módulo del editor debe producir un protocolo idéntico con O2 y sin O2.
#[test]
fn editor_3d_desde_st() {
    native(
        r#"
        importar "editor3d";
        editor3d.iniciar("Nivel"; 1);
        editor3d.entidad("Jugador"; "cubo"; "jugador");
        editor3d.posicion(1.0; 2.0; 3.0);
        editor3d.rotacion(0.0; 0.0; 0.0; 1.0);
        editor3d.escala(1.0; 2.0; 1.0);
        editor3d.terminar();
        "#,
        concat!(
            "PYFORGE_EDITOR3D_V1\nNivel\n1\nJugador\ncubo\njugador\n",
            "1.0\n2.0\n3.0\n0.0\n0.0\n0.0\n1.0\n1.0\n2.0\n1.0\n",
            "FIN_PYFORGE_EDITOR3D_V1\n"
        ),
    );
}

/// Las referencias prestadas no escapan, no se copian ni se confunden con enteros/archivos.
#[test]
fn recursos_rechazos_estaticos() {
    let root = Temp::new();
    let entry = root.0.join("main.st");
    for source in [
        "variable b = recursos.buffer(4);",
        "recursos.buffer(4);",
        "usar b = recursos.buffer(4) {} mostrar(recursos.tamano(b));",
        "usar b = recursos.buffer(4) { variable alias = b; }",
        "usar b = recursos.buffer(4) { b = recursos.buffer(1); }",
        "usar b = recursos.buffer(4) { nativo.cerrar(b); }",
        "usar b = recursos.buffer(4) { mostrar(entero(b)); }",
        "usar b = recursos.buffer(4) { mostrar(b); }",
        "usar b = recursos.buffer(4) { variable xs = [b]; }",
        "usar b = recursos.buffer(4) { recursos.leer(b; 1); }",
        "usar b = recursos.buffer(4) { usar alias = b {} }",
        "recursos.tamano(recursos.buffer(4));",
        "mostrar(tipo(recursos.buffer(4)));",
        "mostrar(booleano(recursos.buffer(4)));",
        "mostrar(recursos.buffer(4) es recursos.buffer(4));",
        "funcion escape(b: buffer_nativo) -> buffer_nativo { retornar b; }",
        "funcion reenviar(b: buffer_nativo) -> buffer_nativo { retornar escape(b); } funcion escape(b: buffer_nativo) -> buffer_nativo { retornar b; }",
        "funcion escape() -> buffer_nativo { usar b = recursos.buffer(4) { retornar b; } }",
        "funcion escape(b: buffer_nativo)->nada { variable copia = b; }",
        "clase Caja { variable datos: buffer_nativo; }",
        "usar b = recursos.buffer(4) { recursos.aux_cerrar_buffer(b); }",
        "usar b = 42 {}",
    ] {
        fs::write(&entry, format!("importar \"recursos\"; importar \"nativo\"; {source}")).unwrap();
        assert!(compile_file(&entry, true).is_err(), "aceptó: {source}");
    }
}

/// Los archivos se cierran sin exponer sus identificadores y una adquisición fallida limpia al propietario exterior.
#[test]
fn recursos_archivos_tipados() {
    let root = Temp::new();
    let entry = root.0.join("main.st");
    let output = root.0.join("programa");
    fs::write(
        &entry,
        r#"
        importar "recursos";
        para i en rango(80) {
            intentar {
                usar datos = recursos.buffer(1) {
                    usar archivo = recursos.abrir_archivo("ausente"; 0) { pasar; }
                }
            } capturar e { pasar; }
        }
        usar archivo = recursos.abrir_archivo("datos.bin"; 2) {
            usar datos = recursos.desde_texto("Hola ñ") {
                mostrar(recursos.escribir_archivo(archivo; datos));
            }
        }
        usar archivo = recursos.abrir_archivo("datos.bin"; 0) {
            mostrar(tipo(archivo));
            usar datos = recursos.leer(archivo; 65536) { mostrar(recursos.texto(datos)); }
            usar fin = recursos.leer(archivo; 1) { mostrar(recursos.tamano(fin)); }
        }
    "#,
    )
    .unwrap();
    for opt in [true, false] {
        pyforge::driver::build_executable(&entry, &output, opt, false).unwrap();
        let result = Command::new(&output).current_dir(&root.0).output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        assert_eq!(result.stdout, "7\narchivo_nativo\nHola ñ\n0\n".as_bytes());
    }
}
