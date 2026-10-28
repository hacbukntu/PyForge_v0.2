//! Léxico exclusivo de ST. Cambia KEYWORDS para elegir tus palabras reservadas.
//! Los nombres se reconocen completos: `siempre` no se divide como `si` + `empre`.
//! Cadenas y comentarios se procesan antes de clasificar palabras.

/// Significado de cada palabra, independiente de cómo se escribe en el archivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Keyword {
    Clase,
    Estructura,
    Implementar,
    Nuevo,
    Yo,
    Intentar,
    Capturar,
    Finalmente,
    Lanzar,
    Desde,
    Como,
    Coincidir,
    Caso,
    Entrada,
    Abrir,
    Escribir,
    Interpolar,
    Buscar,
    Reemplazar,
    Cadena,
    Flotante,
    Tupla,
    Conjunto,
    Nulo,
    Mixto,
    Es,
    Importar,
    Lista,
    Diccionario,
    Agregar,
    Longitud,
    Variable,
    /// Exporta una función o variable de nivel de módulo.
    Publica,
    /// Impide que otros módulos importen una función o variable.
    Privada,
    Mostrar,
    Funcion,
    Retornar,
    Si,
    Sino,
    /// Rama condicional alternativa, equivalente a sino si.
    SinoSi,
    Mientras,
    Para,
    En,
    Rango,
    Romper,
    Continuar,
    Pasar,
    Verdadero,
    Falso,
    Y,
    O,
    No,
    Entero,
    Booleano,
    Nada,
}

/// Diccionario editable de ST: (texto de origen, significado interno).
/// Añadir otro texto con el mismo Keyword crea un alias; quitar el anterior lo elimina.
/// Usa identificadores ASCII y textos únicos. Una operación nueva necesita además
/// una variante de Keyword y una regla en parser: registrarla aquí no basta.
pub const KEYWORDS: &[(&str, Keyword)] = &[
    ("clase", Keyword::Clase),
    ("estructura", Keyword::Estructura),
    ("implementar", Keyword::Implementar),
    ("nuevo", Keyword::Nuevo),
    ("self", Keyword::Yo),
    ("intentar", Keyword::Intentar),
    ("capturar", Keyword::Capturar),
    ("finalmente", Keyword::Finalmente),
    ("lanzar", Keyword::Lanzar),
    ("desde", Keyword::Desde),
    ("como", Keyword::Como),
    ("coincidir", Keyword::Coincidir),
    ("caso", Keyword::Caso),
    ("entrada", Keyword::Entrada),
    ("abrir", Keyword::Abrir),
    ("escribir", Keyword::Escribir),
    ("interpolar", Keyword::Interpolar),
    ("buscar", Keyword::Buscar),
    ("reemplazar", Keyword::Reemplazar),
    ("cadena", Keyword::Cadena),
    ("flotante", Keyword::Flotante),
    ("tupla", Keyword::Tupla),
    ("conjunto", Keyword::Conjunto),
    ("nulo", Keyword::Nulo),
    ("mixto", Keyword::Mixto),
    ("es", Keyword::Es),
    ("importar", Keyword::Importar),
    ("lista", Keyword::Lista),
    ("diccionario", Keyword::Diccionario),
    ("agregar", Keyword::Agregar),
    ("longitud", Keyword::Longitud),
    ("variable", Keyword::Variable),
    ("publica", Keyword::Publica),
    ("privada", Keyword::Privada),
    ("mostrar", Keyword::Mostrar),
    ("funcion", Keyword::Funcion),
    ("retornar", Keyword::Retornar),
    ("si", Keyword::Si),
    ("sino", Keyword::Sino),
    ("sino_si", Keyword::SinoSi),
    ("mientras", Keyword::Mientras),
    ("para", Keyword::Para),
    ("en", Keyword::En),
    ("rango", Keyword::Rango),
    ("romper", Keyword::Romper),
    ("continuar", Keyword::Continuar),
    ("pasar", Keyword::Pasar),
    ("verdadero", Keyword::Verdadero),
    ("falso", Keyword::Falso),
    ("y", Keyword::Y),
    ("o", Keyword::O),
    ("no", Keyword::No),
    ("entero", Keyword::Entero),
    ("booleano", Keyword::Booleano),
    ("nada", Keyword::Nada),
];

/// Palabras contextuales y tipos especiales que el parser reconoce solo donde
/// corresponden. El IDE lee esta tabla para colorear y completar sin impedir que
/// proyectos anteriores tengan una función normal llamada `usar`.
pub const CONTEXTUAL_WORDS: &[(&str, &str)] = &[
    ("usar", "keyword"),
    ("buffer_nativo", "type"),
    ("archivo_nativo", "type"),
];

/// Clasifica una palabra completa; devuelve None si es un identificador de usuario.
pub fn keyword(word: &str) -> Option<Keyword> {
    KEYWORDS
        .iter()
        .find(|(text, _)| *text == word)
        .map(|(_, value)| *value)
}

/// Clases léxicas. Los símbolos conservan su texto para que la gramática sea legible.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Keyword(Keyword),
    Ident(String),
    Float(f64),
    Int(i64),
    Str(String),
    Symbol(String),
    Eof,
}

/// Token y posición de origen; líneas y columnas empiezan en uno.
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: Kind,
    pub line: usize,
    pub column: usize,
}

/// Cursor sobre caracteres Unicode. Los identificadores se restringen a ASCII,
/// pero las cadenas admiten Unicode. pos, line y column avanzan conjuntamente.
struct Lexer {
    chars: Vec<char>,
    pos: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    /// Consume un carácter si existe y mantiene la posición para diagnósticos.
    fn advance(&mut self) -> Option<char> {
        let c = *self.chars.get(self.pos)?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }
        Some(c)
    }

    /// Omite un comentario hasta el primer cierre del mismo formato. No anida
    /// delimitadores ni interpreta escapes; advance conserva líneas y columnas.
    fn block_comment(&mut self, opening: &str, closing: &str) -> Result<(), String> {
        for _ in opening.chars() {
            self.advance();
        }
        let end: Vec<char> = closing.chars().collect();
        while self.pos < self.chars.len() {
            if self.chars[self.pos..].starts_with(&end) {
                for _ in &end {
                    self.advance();
                }
                return Ok(());
            }
            self.advance();
        }
        Err(format!("comentario multilínea sin cerrar: falta {closing}"))
    }

    /// Lee una cadena tras su comilla inicial, respetando escapes y su cierre.
    /// Rechaza saltos de línea literales y escapes desconocidos; usa \n para un salto.
    fn string(&mut self, quote: char) -> Result<String, String> {
        let mut text = String::new();
        while let Some(c) = self.advance() {
            match c {
                c if c == quote => return Ok(text),
                '\n' | '\r' => return Err("cadena sin cerrar antes del fin de línea".into()),
                '\\' => text.push(match self.advance() {
                    Some('n') => '\n',
                    Some('r') => '\r',
                    Some('t') => '\t',
                    Some('\\') => '\\',
                    Some('\'') => '\'',
                    Some('"') => '"',
                    _ => return Err("secuencia de escape no válida".into()),
                }),
                _ => text.push(c),
            }
        }
        Err("cadena sin cerrar".into())
    }

    /// Reconoce un token desde el carácter actual. Lo llama tokenize después
    /// de omitir espacios y comentarios; consume siempre al menos un carácter.
    fn next_kind(&mut self) -> Result<Kind, String> {
        let c = self.advance().ok_or("fin de archivo inesperado")?;
        if c == '"' || c == '\'' {
            return self.string(c).map(Kind::Str);
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let mut word = String::from(c);
            while self
                .chars
                .get(self.pos)
                .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '_')
            {
                word.push(self.advance().unwrap());
            }
            return Ok(keyword(&word)
                .map(Kind::Keyword)
                .unwrap_or(Kind::Ident(word)));
        }
        if c.is_ascii_digit() {
            let mut digits = String::from(c);
            while self.chars.get(self.pos).is_some_and(|c| c.is_ascii_digit()) {
                digits.push(self.advance().unwrap());
            }
            // Un punto seguido de dígito inicia la fracción; e/E introduce exponente.
            // Así se conserva el punto de llamadas de método como valores.agregar(...).
            let mut floating = false;
            if self.chars.get(self.pos) == Some(&'.')
                && self
                    .chars
                    .get(self.pos + 1)
                    .is_some_and(|c| c.is_ascii_digit())
            {
                floating = true;
                digits.push(self.advance().unwrap());
                while self.chars.get(self.pos).is_some_and(|c| c.is_ascii_digit()) {
                    digits.push(self.advance().unwrap());
                }
            }
            if self
                .chars
                .get(self.pos)
                .is_some_and(|c| matches!(c, 'e' | 'E'))
            {
                floating = true;
                digits.push(self.advance().unwrap());
                if self
                    .chars
                    .get(self.pos)
                    .is_some_and(|c| matches!(c, '+' | '-'))
                {
                    digits.push(self.advance().unwrap());
                }
                if !self.chars.get(self.pos).is_some_and(|c| c.is_ascii_digit()) {
                    return Err("el exponente requiere dígitos".into());
                }
                while self.chars.get(self.pos).is_some_and(|c| c.is_ascii_digit()) {
                    digits.push(self.advance().unwrap());
                }
            }
            if floating {
                let value: f64 = digits.parse().map_err(|_| "literal flotante no válido")?;
                if !value.is_finite() {
                    return Err("flotante fuera de rango; usa un valor finito".into());
                }
                return Ok(Kind::Float(value));
            }
            return digits
                .parse()
                .map(Kind::Int)
                .map_err(|_| "entero fuera del rango i64".into());
        }
        let symbol = String::from(c);
        if let Some(next) = self.chars.get(self.pos) {
            let pair = format!("{c}{next}");
            if ["==", "!=", "<=", ">=", "+=", "-=", "*=", "/=", "%=", "->"].contains(&pair.as_str())
            {
                self.advance();
                return Ok(Kind::Symbol(pair));
            }
        }
        if ".[]{}();,:+-*/%<>=!".contains(c) {
            // ! solo tiene significado al formar !=; el parser rechazará ! aislado.
            return Ok(Kind::Symbol(symbol));
        }
        Err(format!("carácter no reconocido: {c:?}"))
    }
}

/// Convierte `source` en tokens con posiciones y Eof final.
/// Omite espacios, comentarios de línea y bloques /* ... */ o de comillas triples.
/// Los errores léxicos incluyen la posición inicial del token problemático.
pub fn tokenize(source: &str) -> Result<Vec<Token>, String> {
    let mut lexer = Lexer {
        chars: source.chars().collect(),
        pos: 0,
        line: 1,
        column: 1,
    };
    let mut tokens = Vec::new();
    while let Some(&c) = lexer.chars.get(lexer.pos) {
        if c.is_whitespace() {
            lexer.advance();
            continue;
        }
        // Los comentarios se reconocen antes de cadenas y del operador división.
        let remaining = &lexer.chars[lexer.pos..];
        let delimiters = if remaining.starts_with(&['/', '*']) {
            Some(("/*", "*/"))
        } else if remaining.starts_with(&['"', '"', '"']) {
            Some(("\"\"\"", "\"\"\""))
        } else {
            None
        };
        if let Some((opening, closing)) = delimiters {
            let (line, column) = (lexer.line, lexer.column);
            lexer
                .block_comment(opening, closing)
                .map_err(|e| super::diagnostic::render(source, line, column, &e))?;
            continue;
        }
        if c == '#' || (c == '/' && lexer.chars.get(lexer.pos + 1) == Some(&'/')) {
            while lexer.chars.get(lexer.pos).is_some_and(|c| *c != '\n') {
                lexer.advance();
            }
            continue;
        }
        let (line, column) = (lexer.line, lexer.column);
        let kind = lexer
            .next_kind()
            .map_err(|e| super::diagnostic::render(source, line, column, &e))?;
        tokens.push(Token { kind, line, column });
    }
    tokens.push(Token {
        kind: Kind::Eof,
        line: lexer.line,
        column: lexer.column,
    });
    Ok(tokens)
}
