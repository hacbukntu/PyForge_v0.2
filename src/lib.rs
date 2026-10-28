//! Biblioteca que conecta las etapas de PyForge.
//! Recorrido de una compilación: bin/pyforgec o bin/pyforge → driver →
//! parser (usa lexer y ast) → semantic → lower → optimizer → backend seleccionado.
//! La IR definida en ir conecta lower y optimizer con `codegen::llvm`, construido
//! sobre LLVM 21 mediante Inkwell y usado en todas las compilaciones.
//! `driver` invoca `as` y `ld` para convertir el ensamblador en un ejecutable y
//! `project` gestiona el manifiesto. El runtime final sigue siendo Linux x86-64.
//! Para estudiar el proyecto, comienza por driver::compile_text y sigue ese orden.
/// Expone el módulo ast; su cabecera describe su papel y conexiones.
pub mod ast;
/// Expone el módulo codegen; su cabecera describe su papel y conexiones.
pub mod codegen;
/// Expone el módulo driver; su cabecera describe su papel y conexiones.
pub mod driver;
/// Expone el módulo ir; su cabecera describe su papel y conexiones.
pub mod ir;
/// Expone el módulo lexer; su cabecera describe su papel y conexiones.
pub mod lexer;
/// Expone el módulo lower; su cabecera describe su papel y conexiones.
pub mod lower;
/// Expone el módulo optimizer; su cabecera describe su papel y conexiones.
pub mod optimizer;
/// Expone el módulo parser; su cabecera describe su papel y conexiones.
pub mod parser;
/// Expone el módulo project; su cabecera describe su papel y conexiones.
pub mod project;
/// Expone el módulo semantic; su cabecera describe su papel y conexiones.
pub mod semantic;

/// Versión compartida por las dos interfaces de línea de comandos.
pub const VERSION: &str = "0.2.0";

/// Entrada del lenguaje ST: léxico y gramática propios, AST y generación compartidos.
pub mod st;

/// Verifica las garantías de flujo de la IR antes de generar código nativo.
pub mod verify;

/// Plantillas de sitios web y foro, conservando el compilador nativo.
pub mod web_project;
