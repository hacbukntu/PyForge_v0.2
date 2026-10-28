//! Entrada independiente del lenguaje ST: texto .st → AST compartido.
//! lexer reconoce palabras/símbolos; parser construye ast::Program directamente.
//! No se genera código Python ni se modifica su parser. driver elige esta entrada
//! por extensión y reutiliza semantic, lower, optimizer y codegen.

/// Palabras reservadas, posiciones de origen y tokenización de ST.
pub mod lexer;
/// Expansión nativa de clases y resolución de sus miembros tipados.
pub mod objects;
/// Gramática de sentencias, bloques y expresiones de ST.
pub mod parser;

/// Analiza `source` como ST y devuelve el AST o un diagnóstico con línea y columna.
/// Es la entrada pública que utiliza driver::compile_st_text.
pub fn parse_source(source: &str) -> Result<crate::ast::Program, String> {
    parser::parse(source)
}

/// Carga de archivos y dependencias ST; separada del análisis de texto en memoria.
pub mod modules;

/// Formato de diagnósticos con línea de código y posición de origen.
pub mod diagnostic;

/// Biblioteca estándar incrustada y firmas de sus primitivas nativas.
pub mod stdlib;
