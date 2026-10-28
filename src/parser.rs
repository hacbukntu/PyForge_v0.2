//! Análisis sintáctico: archivo Python → ast::Program.
//! FileParser reconoce líneas y bloques por sangría; ExprParser organiza los tokens
//! de lexer por precedencia. driver entrega el AST resultante a semantic.
//! Los analizadores consumen cursores mutables y propagan el primer error mediante Result.
use crate::ast::*;
use crate::lexer::{lex_expr, Token};

#[derive(Debug, Clone)]
/// Línea preparada: `no` es el número de origen, indent su sangría en espacios
/// y text el contenido sin sangría ni comentarios exteriores.
struct Line {
    no: usize,
    indent: usize,
    text: String,
}

/// Prepara las líneas de `source` y crea FileParser con el cursor en cero.
/// Devuelve el Program construido por parse_program o el primer error sintáctico.
pub fn parse_source(source: &str) -> Result<Program, String> {
    let lines = preprocess(source)?;
    let mut p = FileParser { lines, pos: 0 };
    p.parse_program()
}

/// Elimina comentarios exteriores, espacios finales y líneas vacías de `source`.
/// Conserva el número original y la sangría de cada línea. Rechaza tabulaciones
/// y sangrías que no sean múltiplos de cuatro, antes de construir `Vec<Line>`.
fn preprocess(source: &str) -> Result<Vec<Line>, String> {
    let mut out = Vec::new();
    for (idx, raw) in source.lines().enumerate() {
        if raw.contains('\t') {
            return Err(format!(
                "line {}: tabs are not supported; use 4 spaces",
                idx + 1
            ));
        }
        let clean = strip_comment(raw).trim_end().to_string();
        if clean.trim().is_empty() {
            continue;
        }
        let indent = clean.chars().take_while(|c| *c == ' ').count();
        if indent % 4 != 0 {
            return Err(format!(
                "line {}: indentation must be a multiple of 4 spaces",
                idx + 1
            ));
        }
        out.push(Line {
            no: idx + 1,
            indent,
            text: clean[indent..].to_string(),
        });
    }
    Ok(out)
}

/// Copia `s` hasta el primer # que esté fuera de comillas.
/// Recuerda escapes y comillas para conservar # dentro de cadenas; char_indices
/// proporciona posiciones válidas en UTF-8. Si no hay comentario copia toda la línea.
fn strip_comment(s: &str) -> String {
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
        } else if c == '\'' || c == '"' {
            quote = Some(c);
        } else if c == '#' {
            return s[..i].to_string();
        }
    }
    s.to_string()
}

/// Estado del analizador del archivo: lines contiene las líneas preparadas
/// y pos señala la siguiente pendiente. Los métodos avanzan ese cursor.
struct FileParser {
    lines: Vec<Line>,
    pos: usize,
}
impl FileParser {
    /// Consume las líneas superiores, separando funciones y sentencias en Program.
    /// Las funciones pasan por parse_function y el resto por parse_block.
    /// Requiere sangría cero y rechaza ramas huérfanas para garantizar progreso.
    fn parse_program(&mut self) -> Result<Program, String> {
        let mut program = Program::default();
        while self.pos < self.lines.len() {
            let line = self.lines[self.pos].clone();
            if line.indent != 0 {
                return self.err(&line, "unexpected indentation at top level");
            }
            if line.text.starts_with("def ") {
                program.functions.push(self.parse_function()?);
            } else {
                let before = self.pos;
                let mut chunk = self.parse_block(0, true)?;
                if self.pos == before {
                    return self.err(&line, "elif/else without a preceding if");
                }
                program.body.append(&mut chunk);
            }
        }
        Ok(program)
    }

    /// Extrae de la cabecera actual el nombre, parámetros y retorno.
    /// Acepta tanto definiciones tipadas como funciones Python normales sin
    /// anotación de parámetros o retorno. Sin anotación se usa entero, como contrato
    /// del subconjunto nativo; nunca se interpreta una referencia como entero dinámico.
    fn parse_function(&mut self) -> Result<FunctionDef, String> {
        let line = self.lines[self.pos].clone();
        let header = line.text.strip_prefix("def ").unwrap();
        if !header.ends_with(':') {
            return self.err(&line, "function header must end with ':'");
        }
        let core = &header[..header.len() - 1];
        let lp = core
            .find('(')
            .ok_or_else(|| format!("line {}: expected '(' in function definition", line.no))?;
        let rp = core
            .rfind(')')
            .ok_or_else(|| format!("line {}: expected ')' in function definition", line.no))?;
        if rp < lp {
            return self.err(&line, "malformed parameter list");
        }
        let name = core[..lp].trim();
        validate_ident(name).map_err(|e| format!("line {}: {e}", line.no))?;
        let params_text = &core[lp + 1..rp];
        let tail = core[rp + 1..].trim();
        let ret = if tail.is_empty() {
            TypeName::Int
        } else if let Some(t) = tail.strip_prefix("->") {
            TypeName::parse(t)
                .ok_or_else(|| format!("line {}: unsupported return annotation", line.no))?
        } else {
            return self.err(&line, "expected '-> TYPE' after parameter list");
        };
        let mut params = Vec::new();
        if !params_text.trim().is_empty() {
            for item in split_top_level(params_text, ',') {
                let item = item.trim();
                if item.is_empty() {
                    continue;
                }
                if let Some(colon) = item.rfind(':') {
                    let pname = item[..colon].trim();
                    let pty = item[colon + 1..].trim();
                    validate_ident(pname).map_err(|e| format!("line {}: {e}", line.no))?;
                    let ty = TypeName::parse(pty).ok_or_else(|| {
                        format!("line {}: unsupported parameter annotation", line.no)
                    })?;
                    params.push(Param {
                        name: pname.to_string(),
                        ty,
                    });
                } else {
                    validate_ident(item).map_err(|e| format!("line {}: {e}", line.no))?;
                    params.push(Param {
                        name: item.to_string(),
                        ty: TypeName::Int,
                    });
                }
            }
        }
        if params.len() > 6 {
            return self.err(&line, "v0.2 supports at most 6 function parameters");
        }
        self.pos += 1;
        let body = self.parse_block(4, false)?;
        if body.is_empty() {
            return self.err(&line, "function body cannot be empty");
        }
        Ok(FunctionDef {
            name: name.to_string(),
            params,
            ret,
            body,
            visibility: Visibility::Public,
        })
    }

    /// Consume sentencias con `indent` espacios y devuelve su lista ordenada.
    /// Se detiene al bajar la sangría o ante elif/else, dejando esa línea pendiente.
    /// Con `stop_on_def` también se detiene antes de una función de nivel superior.
    /// La sangría excesiva es un error; parse_stmt consume cada sentencia aceptada.
    fn parse_block(&mut self, indent: usize, stop_on_def: bool) -> Result<Vec<Stmt>, String> {
        let mut out = Vec::new();
        while self.pos < self.lines.len() {
            let line = self.lines[self.pos].clone();
            if line.indent < indent {
                break;
            }
            if line.indent > indent {
                return self.err(&line, "unexpected indentation");
            }
            if stop_on_def && indent == 0 && line.text.starts_with("def ") {
                break;
            }
            if line.text.starts_with("elif ") || line.text == "else:" {
                break;
            }
            out.push(self.parse_stmt(indent)?);
        }
        Ok(out)
    }

    /// Analiza la línea actual dentro del bloque de sangría `indent` y avanza pos.
    /// Reconoce if, while, for/range, control de bucles, return y asignaciones.
    /// Los cuerpos se leen con cuatro espacios adicionales. Busca asignación aumentada
    /// antes que simple; como último caso intenta una expresión. Devuelve Stmt o error.
    fn parse_stmt(&mut self, indent: usize) -> Result<Stmt, String> {
        let line = self.lines[self.pos].clone();
        let t = line.text.trim();
        if t.starts_with("if ") {
            return self.parse_if(indent);
        }
        if t.starts_with("while ") {
            if !t.ends_with(':') {
                return self.err(&line, "while header must end with ':'");
            }
            let cond =
                parse_expr(&t[6..t.len() - 1]).map_err(|e| format!("line {}: {e}", line.no))?;
            self.pos += 1;
            let body = self.parse_block(indent + 4, false)?;
            if body.is_empty() {
                return self.err(&line, "while body cannot be empty");
            }
            return Ok(Stmt::While { cond, body });
        }
        if t.starts_with("for ") {
            if !t.ends_with(':') {
                return self.err(&line, "for header must end with ':'");
            }
            let core = &t[4..t.len() - 1];
            let in_pos = core
                .find(" in ")
                .ok_or_else(|| format!("line {}: expected 'in' in for loop", line.no))?;
            let var = core[..in_pos].trim();
            validate_ident(var).map_err(|e| format!("line {}: {e}", line.no))?;
            let rhs = core[in_pos + 4..].trim();
            self.pos += 1;
            let body = self.parse_block(indent + 4, false)?;
            if body.is_empty() {
                return self.err(&line, "for body cannot be empty");
            }
            if rhs.starts_with("range(") && rhs.ends_with(')') {
                let inside = &rhs[6..rhs.len() - 1];
                let args = split_top_level(inside, ',');
                let (start, stop) = match args.len() {
                    1 => (
                        Expr::Int(0),
                        parse_expr(args[0].trim()).map_err(|e| format!("line {}: {e}", line.no))?,
                    ),
                    2 => (
                        parse_expr(args[0].trim()).map_err(|e| format!("line {}: {e}", line.no))?,
                        parse_expr(args[1].trim()).map_err(|e| format!("line {}: {e}", line.no))?,
                    ),
                    _ => return self.err(&line, "range() supports 1 or 2 arguments in v0.2"),
                };
                return Ok(Stmt::ForRange {
                    var: var.to_string(),
                    start,
                    stop,
                    body,
                });
            }
            let stop = Expr::Length(Box::new(
                parse_expr(rhs).map_err(|e| format!("line {}: {e}", line.no))?,
            ));
            return Ok(Stmt::ForRange {
                var: var.to_string(),
                start: Expr::Int(0),
                stop,
                body,
            });
        }
        if t == "break" {
            self.pos += 1;
            return Ok(Stmt::Break);
        }
        if t == "continue" {
            self.pos += 1;
            return Ok(Stmt::Continue);
        }
        if t == "pass" {
            self.pos += 1;
            return Ok(Stmt::Pass);
        }
        if t.starts_with("import ") {
            let module = t.strip_prefix("import ").unwrap().trim();
            if module.is_empty() {
                return self.err(&line, "empty import is not allowed");
            }
            for part in split_top_level(module, ',') {
                let item = part.trim();
                if item.is_empty() {
                    continue;
                }
                let name = item.split_whitespace().next().unwrap_or(item);
                let name = name.strip_suffix(" as").unwrap_or(name).trim();
                if name.is_empty() || !is_supported_python_module(name) {
                    return self.err(
                        &line,
                        &format!("Python import '{name}' is not supported in v0.2"),
                    );
                }
            }
            self.pos += 1;
            return Ok(Stmt::Pass);
        }
        if t.starts_with("from ") {
            let rest = t.strip_prefix("from ").unwrap().trim();
            let Some((module, items)) = rest.split_once(" import ") else {
                return self.err(&line, "expected 'import ...' after module name");
            };
            let module = module.trim();
            if !is_supported_python_module(module) {
                return self.err(
                    &line,
                    &format!("Python import '{module}' is not supported in v0.2"),
                );
            }
            for item in split_top_level(items, ',') {
                let item = item.trim();
                if item.is_empty() {
                    continue;
                }
                let name = item.split_whitespace().next().unwrap_or(item).trim();
                if name.is_empty() {
                    return self.err(&line, "invalid import list");
                }
            }
            self.pos += 1;
            return Ok(Stmt::Pass);
        }
        if t == "return" {
            self.pos += 1;
            return Ok(Stmt::Return(None));
        }
        if let Some(rest) = t.strip_prefix("return ") {
            let expr = parse_expr(rest).map_err(|e| format!("line {}: {e}", line.no))?;
            self.pos += 1;
            return Ok(Stmt::Return(Some(expr)));
        }
        if let Some((name, op, rhs)) = split_aug_assignment(t) {
            validate_ident(name).map_err(|e| format!("line {}: {e}", line.no))?;
            let expr = parse_expr(rhs).map_err(|e| format!("line {}: {e}", line.no))?;
            self.pos += 1;
            return Ok(Stmt::AugAssign {
                name: name.to_string(),
                op,
                expr,
            });
        }
        if let Some((lhs, rhs)) = split_plain_assignment(t) {
            let (name, annotation) = if let Some(colon) = lhs.find(':') {
                let n = lhs[..colon].trim();
                let ty_s = lhs[colon + 1..].trim();
                let ty = TypeName::parse(ty_s).ok_or_else(|| {
                    format!("line {}: unsupported type annotation: {ty_s}", line.no)
                })?;
                (n, Some(ty))
            } else {
                (lhs.trim(), None)
            };
            validate_ident(name).map_err(|e| format!("line {}: {e}", line.no))?;
            let expr = parse_expr(rhs).map_err(|e| format!("line {}: {e}", line.no))?;
            self.pos += 1;
            return Ok(Stmt::Assign {
                name: name.to_string(),
                annotation,
                expr,
                binding: BindingKind::DefineOrAssign,
                visibility: Visibility::Private,
            });
        }
        let expr = parse_expr(t).map_err(|e| format!("line {}: {e}", line.no))?;
        self.pos += 1;
        Ok(Stmt::Expr(expr))
    }

    /// Agrupa el if actual, sus elif y el posible else a la sangría `indent`.
    /// Guarda pares (condición, cuerpo), exige dos puntos y cuerpos no vacíos.
    /// Deja pos en la primera línea ajena a esa cadena y devuelve Stmt::If.
    fn parse_if(&mut self, indent: usize) -> Result<Stmt, String> {
        let first = self.lines[self.pos].clone();
        let mut branches = Vec::new();
        let mut else_body = Vec::new();
        loop {
            let line = self.lines[self.pos].clone();
            let t = line.text.trim();
            if t.starts_with("if ") || t.starts_with("elif ") {
                if !t.ends_with(':') {
                    return self.err(&line, "if/elif header must end with ':'");
                }
                let start = if t.starts_with("if ") { 3 } else { 5 };
                let cond = parse_expr(&t[start..t.len() - 1])
                    .map_err(|e| format!("line {}: {e}", line.no))?;
                self.pos += 1;
                let body = self.parse_block(indent + 4, false)?;
                if body.is_empty() {
                    return self.err(&line, "if/elif body cannot be empty");
                }
                branches.push((cond, body));
            } else {
                break;
            }
            if self.pos >= self.lines.len() {
                break;
            }
            let next = self.lines[self.pos].clone();
            if next.indent != indent || !(next.text.starts_with("elif ") || next.text == "else:") {
                break;
            }
            if next.text == "else:" {
                self.pos += 1;
                else_body = self.parse_block(indent + 4, false)?;
                if else_body.is_empty() {
                    return self.err(&next, "else body cannot be empty");
                }
                break;
            }
        }
        if branches.is_empty() {
            return self.err(&first, "internal parser error: empty if");
        }
        Ok(Stmt::If {
            branches,
            else_body,
        })
    }

    /// Construye un Err con el número original de `line` y el mensaje `msg`.
    /// El tipo genérico T permite usarlo desde métodos con distintos resultados.
    fn err<T>(&self, line: &Line, msg: &str) -> Result<T, String> {
        Err(format!("line {}: {msg}", line.no))
    }
}

/// Valida que `s` comience por letra ASCII o guion bajo y continúe con
/// letras ASCII, dígitos o guiones bajos. Devuelve () o error; comprueba la forma,
/// pero no excluye todas las palabras reservadas del lenguaje.
fn validate_ident(s: &str) -> Result<(), String> {
    let mut it = s.chars();
    match it.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return Err(format!("invalid identifier: {s}")),
    }
    if !it.all(|c| c.is_ascii_alphanumeric() || c == '_') {
        return Err(format!("invalid identifier: {s}"));
    }
    Ok(())
}

fn is_supported_python_module(module: &str) -> bool {
    matches!(module, "math")
}

/// Divide `s` por `sep` fuera de paréntesis y cadenas, respetando escapes.
/// Devuelve fragmentos propios sin espacios exteriores. La profundidad evita
/// separar argumentos de llamadas anidadas; no valida todos los delimitadores.
fn split_top_level(s: &str, sep: char) -> Vec<String> {
    let mut out = Vec::new();
    let mut start = 0usize;
    let mut depth = 0i32;
    let mut quote = None;
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if c == '\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if let Some(q) = quote {
            if c == q {
                quote = None;
            }
            continue;
        }
        if c == '\'' || c == '"' {
            quote = Some(c);
            continue;
        }
        match c {
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => {}
        }
        if c == sep && depth == 0 {
            out.push(s[start..i].trim().to_string());
            start = i + c.len_utf8();
        }
    }
    out.push(s[start..].trim().to_string());
    out
}

/// Busca +=, -=, *=, /= o %= en `s` mediante find_top_level.
/// Devuelve vistas del nombre y expresión derecha junto a AugOp, o None.
/// La validación de esas partes corresponde después a parse_stmt.
fn split_aug_assignment(s: &str) -> Option<(&str, AugOp, &str)> {
    for (needle, op) in [
        ("+=", AugOp::Add),
        ("-=", AugOp::Sub),
        ("*=", AugOp::Mul),
        ("/=", AugOp::Div),
        ("%=", AugOp::Mod),
    ] {
        if let Some(i) = find_top_level(s, needle) {
            return Some((s[..i].trim(), op, s[i + 2..].trim()));
        }
    }
    None
}

/// Busca un = exterior a comillas y paréntesis sin confundirlo con
/// ==, !=, <= o >=. Devuelve vistas recortadas de izquierda y derecha, o None.
/// Recorre bytes porque los delimitadores reconocidos son ASCII.
fn split_plain_assignment(s: &str) -> Option<(&str, &str)> {
    let bytes = s.as_bytes();
    let mut depth = 0i32;
    let mut quote = 0u8;
    let mut esc = false;
    for i in 0..bytes.len() {
        let b = bytes[i];
        if esc {
            esc = false;
            continue;
        }
        if quote != 0 {
            if b == b'\\' {
                esc = true;
            } else if b == quote {
                quote = 0;
            }
            continue;
        }
        if b == b'\'' || b == b'"' {
            quote = b;
            continue;
        }
        if b == b'(' {
            depth += 1;
            continue;
        }
        if b == b')' {
            depth -= 1;
            continue;
        }
        if b == b'=' && depth == 0 {
            let prev = if i > 0 { bytes[i - 1] } else { 0 };
            let next = if i + 1 < bytes.len() { bytes[i + 1] } else { 0 };
            if prev != b'=' && prev != b'!' && prev != b'<' && prev != b'>' && next != b'=' {
                return Some((s[..i].trim(), s[i + 1..].trim()));
            }
        }
    }
    None
}

/// Busca `needle` dentro de `s` fuera de cadenas y paréntesis.
/// Devuelve el índice en bytes de la primera coincidencia o None. Mantiene
/// profundidad, comilla activa y escapes; lo usa la asignación aumentada.
fn find_top_level(s: &str, needle: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    let n = needle.as_bytes();
    let mut depth = 0i32;
    let mut quote = 0u8;
    let mut esc = false;
    let mut i = 0;
    while i + n.len() <= bytes.len() {
        let b = bytes[i];
        if esc {
            esc = false;
            i += 1;
            continue;
        }
        if quote != 0 {
            if b == b'\\' {
                esc = true;
            } else if b == quote {
                quote = 0;
            }
            i += 1;
            continue;
        }
        if b == b'\'' || b == b'"' {
            quote = b;
            i += 1;
            continue;
        }
        if b == b'(' {
            depth += 1;
            i += 1;
            continue;
        }
        if b == b')' {
            depth -= 1;
            i += 1;
            continue;
        }
        if depth == 0 && &bytes[i..i + n.len()] == n {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// Pide tokens a lex_expr, inicia ExprParser y llama a parse_or.
/// Exige Eof al terminar para rechazar texto sobrante. Devuelve Expr o error;
/// los nombres y tipos se comprobarán en semantic.
pub fn parse_expr(text: &str) -> Result<Expr, String> {
    let toks = lex_expr(text)?;
    let mut p = ExprParser { toks, pos: 0 };
    let e = p.parse_or()?;
    if !matches!(p.peek(), Token::Eof) {
        return Err(format!("unexpected token: {:?}", p.peek()));
    }
    Ok(e)
}

/// Estado del analizador de expresiones: toks termina en Eof y pos señala
/// el siguiente token. Cada nivel de precedencia consume parte de la secuencia.
struct ExprParser {
    toks: Vec<Token>,
    pos: usize,
}
impl ExprParser {
    /// Devuelve una referencia al token actual sin avanzar pos.
    /// Se usa para decidir qué regla aplicar; presupone un índice válido.
    fn peek(&self) -> &Token {
        &self.toks[self.pos]
    }
    /// Clona el token actual y avanza pos una posición.
    /// Separa el valor consumido del estado mutable y presupone un cursor válido.
    fn next(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        self.pos += 1;
        t
    }
    /// Lee operandos de parse_and unidos por or y construye nodos binarios
    /// por la izquierda. Es el nivel de menor precedencia de esta implementación.
    /// El cortocircuito se implementa más tarde en lower.
    fn parse_or(&mut self) -> Result<Expr, String> {
        let mut e = self.parse_and()?;
        while matches!(self.peek(), Token::Or) {
            self.next();
            let r = self.parse_and()?;
            e = Expr::Binary {
                op: BinOp::Or,
                left: Box::new(e),
                right: Box::new(r),
            }
        }
        Ok(e)
    }
    /// Lee comparaciones unidas por and y las agrupa por la izquierda.
    /// Da a and mayor precedencia que or; propaga errores de parse_cmp.
    fn parse_and(&mut self) -> Result<Expr, String> {
        let mut e = self.parse_cmp()?;
        while matches!(self.peek(), Token::And) {
            self.next();
            let r = self.parse_cmp()?;
            e = Expr::Binary {
                op: BinOp::And,
                left: Box::new(e),
                right: Box::new(r),
            }
        }
        Ok(e)
    }
    /// Construye sumas y restas por la izquierda con operandos de parse_mul.
    /// Así multiplicaciones y divisiones se agrupan antes que + y -.
    fn parse_add(&mut self) -> Result<Expr, String> {
        let mut e = self.parse_mul()?;
        loop {
            let op = match self.peek() {
                Token::Plus => BinOp::Add,
                Token::Minus => BinOp::Sub,
                _ => break,
            };
            self.next();
            let r = self.parse_mul()?;
            e = Expr::Binary {
                op,
                left: Box::new(e),
                right: Box::new(r),
            }
        }
        Ok(e)
    }
    /// Construye *, / y % por la izquierda con operandos de parse_unary.
    /// Solo analiza sintaxis; el significado de división entera viene después.
    fn parse_mul(&mut self) -> Result<Expr, String> {
        let mut e = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star => BinOp::Mul,
                Token::Slash => BinOp::Div,
                Token::Percent => BinOp::Mod,
                _ => break,
            };
            self.next();
            let r = self.parse_unary()?;
            e = Expr::Binary {
                op,
                left: Box::new(e),
                right: Box::new(r),
            }
        }
        Ok(e)
    }
    /// Consume los prefijos -, + y not y analiza recursivamente su operando.
    /// Sin prefijo delega en parse_primary. Actualmente not tiene aquí la misma
    /// precedencia alta que los signos, distinta de la de Python.
    fn parse_unary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Token::Minus => {
                self.next();
                Ok(Expr::Unary {
                    op: UnaryOp::Neg,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            Token::Plus => {
                self.next();
                Ok(Expr::Unary {
                    op: UnaryOp::Pos,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            Token::Not => {
                self.next();
                Ok(Expr::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(self.parse_unary()?),
                })
            }
            _ => self.parse_primary(),
        }
    }
    /// Reconoce literales, variables, llamadas, índices y colecciones Python.
    /// Tras un nombre, ( inicia una llamada con argumentos separados por comas,
    /// analizados con parse_or. También soporta acceso por índice y atributo
    /// para mantener compatibilidad con la sintaxis oficial de Python.
    fn parse_primary(&mut self) -> Result<Expr, String> {
        let mut expr = match self.next() {
            Token::Int(n) => Expr::Int(n),
            Token::Float(n) => Expr::Float(n),
            Token::Str(s) => Expr::Str(s),
            Token::True => Expr::Bool(true),
            Token::False => Expr::Bool(false),
            Token::None => Expr::Null,
            Token::Ident(name) => {
                if matches!(self.peek(), Token::LParen) {
                    self.next();
                    let mut args = Vec::new();
                    if !matches!(self.peek(), Token::RParen) {
                        loop {
                            args.push(self.parse_or()?);
                            if matches!(self.peek(), Token::Comma) {
                                self.next();
                                continue;
                            }
                            break;
                        }
                    }
                    if !matches!(self.next(), Token::RParen) {
                        return Err("expected ')'".into());
                    }
                    Expr::Call { name, args }
                } else {
                    Expr::Var(name)
                }
            }
            Token::LParen => {
                let e = self.parse_or()?;
                if !matches!(self.next(), Token::RParen) {
                    return Err("expected ')'".into());
                }
                e
            }
            Token::LBracket => {
                let mut values = Vec::new();
                if !matches!(self.peek(), Token::RBracket) {
                    loop {
                        values.push(self.parse_or()?);
                        if matches!(self.peek(), Token::Comma) {
                            self.next();
                            continue;
                        }
                        break;
                    }
                }
                if !matches!(self.next(), Token::RBracket) {
                    return Err("expected ']'".into());
                }
                Expr::List(values)
            }
            Token::LBrace => {
                let mut values = Vec::new();
                if !matches!(self.peek(), Token::RBrace) {
                    loop {
                        let key = self.parse_or()?;
                        if !matches!(self.next(), Token::Colon) {
                            return Err("expected ':' in dict literal".into());
                        }
                        let value = self.parse_or()?;
                        values.push((key, value));
                        if matches!(self.peek(), Token::Comma) {
                            self.next();
                            continue;
                        }
                        break;
                    }
                }
                if !matches!(self.next(), Token::RBrace) {
                    return Err("expected '}'".into());
                }
                Expr::Dict(values)
            }
            t => return Err(format!("unexpected token in expression: {t:?}")),
        };

        loop {
            match self.peek() {
                Token::LBracket => {
                    self.next();
                    let key = self.parse_or()?;
                    if !matches!(self.next(), Token::RBracket) {
                        return Err("expected ']'".into());
                    }
                    expr = Expr::Index {
                        collection: Box::new(expr),
                        key: Box::new(key),
                    };
                }
                Token::Dot => {
                    self.next();
                    let name = match self.next() {
                        Token::Ident(name) => name,
                        t => return Err(format!("expected attribute name, got {t:?}")),
                    };
                    if matches!(self.peek(), Token::LParen) {
                        self.next();
                        let mut args = Vec::new();
                        if !matches!(self.peek(), Token::RParen) {
                            loop {
                                args.push(self.parse_or()?);
                                if matches!(self.peek(), Token::Comma) {
                                    self.next();
                                    continue;
                                }
                                break;
                            }
                        }
                        if !matches!(self.next(), Token::RParen) {
                            return Err("expected ')'".into());
                        }
                        expr = Expr::MethodCall {
                            object: Box::new(expr),
                            name,
                            args,
                        };
                    } else {
                        expr = Expr::Member {
                            object: Box::new(expr),
                            name,
                        };
                    }
                }
                _ => break,
            }
        }
        Ok(expr)
    }

    /// Lee comparaciones y soporta `in`, `not in`, `is`, `is not` y comparaciones estándar.
    fn parse_cmp(&mut self) -> Result<Expr, String> {
        let mut e = self.parse_add()?;
        loop {
            if matches!(self.peek(), Token::In) {
                self.next();
                let rhs = self.parse_add()?;
                let lhs = e.clone();
                e = Expr::Call {
                    name: "__contains__".to_string(),
                    args: vec![lhs, rhs],
                };
                continue;
            }
            if matches!(self.peek(), Token::Not) {
                self.next();
                if matches!(self.peek(), Token::In) {
                    self.next();
                    let rhs = self.parse_add()?;
                    let lhs = e.clone();
                    let contains = Expr::Call {
                        name: "__contains__".to_string(),
                        args: vec![lhs, rhs],
                    };
                    e = Expr::Unary {
                        op: UnaryOp::Not,
                        expr: Box::new(contains),
                    };
                    continue;
                }
                return Err("expected 'in' after 'not'".into());
            }
            let op = match self.peek() {
                Token::EqEq => Some(BinOp::Eq),
                Token::Ne => Some(BinOp::Ne),
                Token::Lt => Some(BinOp::Lt),
                Token::Le => Some(BinOp::Le),
                Token::Gt => Some(BinOp::Gt),
                Token::Ge => Some(BinOp::Ge),
                Token::Is => Some(BinOp::Is),
                _ => None,
            };
            if let Some(op) = op {
                self.next();
                if op == BinOp::Is && matches!(self.peek(), Token::Not) {
                    self.next();
                    let r = self.parse_add()?;
                    e = Expr::Unary {
                        op: UnaryOp::Not,
                        expr: Box::new(Expr::Binary {
                            op: BinOp::Is,
                            left: Box::new(e),
                            right: Box::new(r),
                        }),
                    };
                    continue;
                }
                let r = self.parse_add()?;
                e = Expr::Binary {
                    op,
                    left: Box::new(e),
                    right: Box::new(r),
                };
                continue;
            }
            break;
        }
        Ok(e)
    }
}
