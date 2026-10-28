//! Presentación compartida de errores léxicos y sintácticos de ST.
//! Los tokens conservan coordenadas reales; nunca se inventa una ubicación semántica.

/// Muestra la línea original y un indicador, expandiendo tabulaciones a cuatro columnas.
/// El cursor usa caracteres Unicode como el lexer; no bytes, para no desplazar acentos.
/// El llamador añade el archivo cuando lo conoce (por ejemplo, el cargador de módulos).
pub fn render(source: &str, line: usize, column: usize, message: &str) -> String {
    let original = source.split('\n').nth(line.saturating_sub(1)).unwrap_or("");
    let prefix = original
        .chars()
        .take(column.saturating_sub(1))
        .collect::<String>();
    let width = prefix.replace('\t', "    ").chars().count();
    let number_width = line.to_string().len();
    format!(
        "ST línea {line}, columna {column}: {message}\n{blank} |\n{line} | {text}\n{blank} | {padding}^",
        blank = " ".repeat(number_width),
        text = original.replace('\t', "    "),
        padding = " ".repeat(width),
    )
}
