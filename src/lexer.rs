//! Análisis léxico de expresiones: texto → tokens.
//! parser::parse_expr llama a lex_expr; la sangría y las sentencias se procesan en parser.
//! Este módulo reconoce símbolos, pero no resuelve nombres ni verifica tipos.
#[derive(Debug, Clone, PartialEq)]
/// Elemento léxico: Int, Str e Ident llevan datos; los demás representan símbolos
/// o palabras booleanas. Eof indica el final de la expresión para el parser.
pub enum Token {
    Int(i64),
    Float(f64),
    Str(String),
    Ident(String),
    True,
    False,
    None,
    And,
    Or,
    Not,
    Is,
    In,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    EqEq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Comma,
    Colon,
    Dot,
    Eof,
}

/// Tokeniza la expresión `input` y devuelve `Vec<Token>` terminado en Eof.
/// Agrupa enteros i64 (elimina guiones bajos), cadenas con escapes, identificadores
/// ASCII, palabras booleanas y operadores. Busca operadores dobles antes que simples.
/// Recorre caracteres Unicode para conservar el texto de las cadenas. Devuelve error
/// ante enteros fuera de rango, cadenas sin cerrar o caracteres no admitidos.
pub fn lex_expr(input: &str) -> Result<Vec<Token>, String> {
    let chars: Vec<char> = input.chars().collect();
    let mut i = 0usize;
    let mut out = Vec::new();
    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        // Agrupa el número completo antes de convertirlo a i64.
        if c.is_ascii_digit() {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '_') {
                i += 1;
            }
            let mut real = false;
            if i < chars.len() && chars[i] == '.' {
                real = true;
                i += 1;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            if i < chars.len() && matches!(chars[i], 'e' | 'E') {
                real = true;
                i += 1;
                if i < chars.len() && matches!(chars[i], '+' | '-') {
                    i += 1;
                }
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
            }
            let s: String = chars[start..i].iter().filter(|&&x| x != '_').collect();
            if real {
                let value = s
                    .parse::<f64>()
                    .map_err(|_| format!("invalid float: {s}"))?;
                if !value.is_finite() {
                    return Err("float out of range".into());
                }
                out.push(Token::Float(value));
            } else {
                let n = s
                    .parse::<i64>()
                    .map_err(|_| format!("integer out of range: {s}"))?;
                out.push(Token::Int(n));
            }
            continue;
        }
        if c == '\'' || c == '"' {
            let quote = c;
            i += 1;
            let mut s = String::new();
            let mut closed = false;
            while i < chars.len() {
                let ch = chars[i];
                if ch == quote {
                    i += 1;
                    closed = true;
                    break;
                }
                if ch == '\\' {
                    i += 1;
                    if i >= chars.len() {
                        return Err("unterminated escape sequence".into());
                    }
                    let esc = match chars[i] {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '\\' => '\\',
                        '\'' => '\'',
                        '"' => '"',
                        other => other,
                    };
                    s.push(esc);
                    i += 1;
                } else {
                    s.push(ch);
                    i += 1;
                }
            }
            if !closed {
                return Err("unterminated string literal".into());
            }
            out.push(Token::Str(s));
            continue;
        }
        // Reconoce el nombre completo y luego distingue palabras reservadas.
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let s: String = chars[start..i].iter().collect();
            out.push(match s.as_str() {
                "True" => Token::True,
                "False" => Token::False,
                "None" => Token::None,
                "and" => Token::And,
                "or" => Token::Or,
                "not" => Token::Not,
                "is" => Token::Is,
                "in" => Token::In,
                _ => Token::Ident(s),
            });
            continue;
        }
        // Prueba operadores de dos caracteres antes de los simples.
        let two = if i + 1 < chars.len() {
            Some((c, chars[i + 1]))
        } else {
            None
        };
        if let Some((a, b)) = two {
            let tok = match (a, b) {
                ('=', '=') => Some(Token::EqEq),
                ('!', '=') => Some(Token::Ne),
                ('<', '=') => Some(Token::Le),
                ('>', '=') => Some(Token::Ge),
                _ => None,
            };
            if let Some(t) = tok {
                out.push(t);
                i += 2;
                continue;
            }
        }
        let tok = match c {
            '+' => Token::Plus,
            '-' => Token::Minus,
            '*' => Token::Star,
            '/' => Token::Slash,
            '%' => Token::Percent,
            '<' => Token::Lt,
            '>' => Token::Gt,
            '(' => Token::LParen,
            ')' => Token::RParen,
            '[' => Token::LBracket,
            ']' => Token::RBracket,
            '{' => Token::LBrace,
            '}' => Token::RBrace,
            ',' => Token::Comma,
            ':' => Token::Colon,
            '.' => Token::Dot,
            _ => return Err(format!("unexpected character in expression: {c:?}")),
        };
        out.push(tok);
        i += 1;
    }
    // El marcador permite al parser detectar texto sobrante al terminar.
    out.push(Token::Eof);
    Ok(out)
}
