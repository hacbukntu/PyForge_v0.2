//! Gramática de ST: tokens → AST, sin pasar por texto o intérprete Python.
//! statement controla las sentencias; block usa llaves; expression define precedencia.
//! Para modificar la sintaxis, conserva el significado de los nodos AST producidos.
use super::lexer::{tokenize, Keyword as K, Kind, Token};
use crate::ast::*;

/// Analiza el archivo completo. Solo permite funciones en el nivel superior.
/// El cursor siempre consume un token o devuelve un error, incluso ante EOF.
pub fn parse(source: &str) -> Result<Program, String> {
    let (mut program, imports) = parse_module(source)?;
    if !imports.is_empty() {
        return Err("Las importaciones requieren un archivo de origen; usa compile_file".into());
    }
    super::modules::resolve_type_queries(&mut program)?;
    Ok(program)
}

/// Lee las importaciones iniciales y el AST local. La carga de rutas corresponde
/// a modules::load; aquí solo se comprueba la gramática y se conservan los nombres.
pub fn parse_module(source: &str) -> Result<(Program, Vec<super::modules::Import>), String> {
    let tokens = tokenize(source)?;
    let mut classes = std::collections::HashMap::new();
    static NEXT_CLASS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    for pair in tokens.windows(2) {
        if matches!(pair[0].kind, Kind::Keyword(K::Clase | K::Estructura)) {
            if let Kind::Ident(name) = &pair[1].kind {
                let id = NEXT_CLASS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                if classes.insert(name.clone(), id).is_some() {
                    return Err(format!("clase o estructura duplicada: {name}"));
                }
            }
        }
    }
    let mut parser = Parser {
        source,
        tokens,
        classes,
        in_method: false,
        pos: 0,
        in_function: false,
    };
    let mut imports = Vec::new();
    while matches!(
        parser.peek(),
        Kind::Keyword(K::Importar) | Kind::Keyword(K::Desde)
    ) {
        let selective = parser.take() == Kind::Keyword(K::Desde);
        let Kind::Str(path) = parser.take() else {
            return Err(parser.error("la importación requiere una ruta entre comillas"));
        };
        let mut import = super::modules::Import {
            path,
            alias: None,
            names: Vec::new(),
        };
        if selective {
            if !parser.eat_keyword(K::Importar) {
                return Err(parser.error("se esperaba importar después de la ruta"));
            }
            loop {
                let name = parser.identifier()?;
                let alias = if parser.eat_keyword(K::Como) {
                    parser.identifier()?
                } else {
                    name.clone()
                };
                import.names.push((name, alias));
                if !parser.eat(",") {
                    break;
                }
            }
        } else if parser.eat_keyword(K::Como) {
            import.alias = Some(parser.identifier()?);
        }
        parser.expect(";")?;
        imports.push(import);
    }
    let mut program = Program::default();
    while parser.peek() != &Kind::Eof {
        // Los modificadores se consumen antes de decidir la clase de declaración.
        // Se limitan al nivel del módulo para que una variable local no parezca una API.
        let visibility = parser.take_visibility();
        if parser.eat_keyword(K::Clase) || parser.eat_keyword(K::Estructura) {
            if visibility.is_some() {
                return Err(parser.error("publica/privada solo precede a funcion o variable"));
            }
            program.functions.extend(parser.class()?);
        } else if parser.eat_keyword(K::Implementar) {
            if visibility.is_some() {
                return Err(parser.error("publica/privada solo precede a funcion o variable"));
            }
            let name = parser.identifier()?;
            let id = *parser.classes.get(&name).ok_or_else(|| {
                parser.error("implementar requiere una clase declarada en este archivo")
            })?;
            parser.expect("{")?;
            while !parser.eat("}") {
                if !parser.eat_keyword(K::Funcion) {
                    return Err(parser.error("implementar solo admite funciones"));
                }
                program.functions.push(parser.method(id)?);
            }
        } else if parser.eat_keyword(K::Funcion) {
            program
                .functions
                .push(parser.function(visibility.unwrap_or(Visibility::Public))?);
        } else if parser.eat_keyword(K::Variable) {
            program
                .body
                .push(parser.variable_declaration(visibility.unwrap_or(Visibility::Private))?);
        } else if visibility.is_some() {
            return Err(parser.error("publica/privada requiere funcion o variable"));
        } else {
            program.body.push(parser.statement()?);
        }
    }
    Ok((program, imports))
}

/// Cursor sintáctico. in_function impide retornar desde el nivel superior.
struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    pos: usize,
    in_function: bool,
    in_method: bool,
    classes: std::collections::HashMap<String, usize>,
}
impl Parser<'_> {
    /// Consume como máximo un modificador de acceso.
    fn take_visibility(&mut self) -> Option<Visibility> {
        if self.eat_keyword(K::Publica) {
            Some(Visibility::Public)
        } else if self.eat_keyword(K::Privada) {
            Some(Visibility::Private)
        } else {
            None
        }
    }

    /// Lee campos obligatorios y métodos. Las estructuras comparten la semántica
    /// nominal de las clases; no implican copia por valor ni herencia.
    fn class(&mut self) -> Result<Vec<FunctionDef>, String> {
        let name = self.identifier()?;
        let id = self.classes[&name];
        let mut fields: Vec<Param> = Vec::new();
        let mut methods = Vec::new();
        self.expect("{")?;
        while !self.eat("}") {
            if self.eat_keyword(K::Variable) {
                let field = self.identifier()?;
                if fields.iter().any(|f| f.name == field) {
                    return Err(self.error("campo duplicado"));
                }
                self.expect(":")?;
                let ty = self.type_name(false)?;
                if ty.is_resource() {
                    return Err(self.error(
                        "un recurso no puede almacenarse en un campo; préstalo como parámetro dentro de usar"
                    ));
                }
                self.expect(";")?;
                fields.push(Param { name: field, ty });
                if fields.len() > 6 {
                    return Err(self.error("el constructor admite hasta seis campos"));
                }
            } else if self.eat_keyword(K::Funcion) {
                methods.push(self.method(id)?);
            } else {
                return Err(self.error("se esperaba variable campo: tipo; o funcion"));
            }
        }
        let mut functions = super::objects::define(name, id, fields);
        functions.extend(methods);
        Ok(functions)
    }

    /// Añade el receptor implícito yo y un símbolo privado único a cada método.
    fn method(&mut self, id: usize) -> Result<FunctionDef, String> {
        self.in_method = true;
        let mut function = self.function(Visibility::Private)?;
        self.in_method = false;
        if function.params.len() > 5 {
            return Err(self.error("un método admite cinco argumentos además de self/yo"));
        }
        function.name = super::objects::symbol(id, "method", &function.name);
        function.params.insert(
            0,
            Param {
                name: "yo".into(),
                ty: TypeName::Object(id),
            },
        );
        Ok(function)
    }

    /// Consulta el token actual; Eof permanece como centinela final.
    fn peek(&self) -> &Kind {
        &self.tokens[self.pos].kind
    }

    /// Produce un diagnóstico en la posición actual sin modificar el cursor.
    fn error(&self, message: &str) -> String {
        let t = &self.tokens[self.pos];
        let found = match &t.kind {
            Kind::Eof => "fin del archivo".to_string(),
            Kind::Ident(name) | Kind::Symbol(name) => format!("'{name}'"),
            Kind::Keyword(word) => super::lexer::KEYWORDS
                .iter()
                .find(|(_, meaning)| meaning == word)
                .map(|(name, _)| format!("palabra reservada '{name}'"))
                .unwrap_or_else(|| "palabra reservada".into()),
            Kind::Float(value) => format!("flotante {value}"),
            Kind::Int(value) => format!("entero {value}"),
            Kind::Str(_) => "cadena de texto".into(),
        };
        super::diagnostic::render(
            self.source,
            t.line,
            t.column,
            &format!("{message}; encontrado: {found}"),
        )
    }

    /// Consume y devuelve el token actual, pero nunca avanza más allá de Eof.
    fn take(&mut self) -> Kind {
        let kind = self.peek().clone();
        if kind != Kind::Eof {
            self.pos += 1;
        }
        kind
    }

    /// Consume la palabra indicada si coincide; sirve para reglas opcionales.
    fn eat_keyword(&mut self, keyword: K) -> bool {
        if self.peek() == &Kind::Keyword(keyword) {
            self.take();
            true
        } else {
            false
        }
    }

    /// Consume el símbolo indicado si coincide; no cambia nada si es otro token.
    fn eat(&mut self, symbol: &str) -> bool {
        if self.peek() == &Kind::Symbol(symbol.into()) {
            self.take();
            true
        } else {
            false
        }
    }

    /// Exige un delimitador u operador y devuelve un error si falta.
    fn expect(&mut self, symbol: &str) -> Result<(), String> {
        if self.eat(symbol) {
            Ok(())
        } else {
            Err(self.error(&format!("se esperaba '{symbol}'")))
        }
    }

    /// Lee un nombre de usuario. Excluye los nombres internos del AST y generador.
    /// Las palabras de KEYWORDS nunca llegan como Ident y no pueden ser variables.
    fn identifier(&mut self) -> Result<String, String> {
        match self.peek().clone() {
            Kind::Ident(name) if name != "print" && !name.starts_with("__") => {
                self.take();
                Ok(name)
            }
            _ => Err(self.error("se esperaba un identificador no reservado")),
        }
    }

    /// Lee un tipo declarado por el usuario.
    ///
    /// `mixto` es deliberadamente un tipo interno de elementos: solo puede
    /// aparecer dentro de `lista[mixto]`. Una variable dinámica aislada
    /// necesitaría que todas las operaciones trabajasen con valores
    /// etiquetados; limitarlo a listas conserva el tipado estático del resto
    /// del lenguaje y evita interpretar por error un puntero como un entero.
    fn type_name(&mut self, allow_void: bool) -> Result<TypeName, String> {
        self.type_name_inner(allow_void, false)
    }

    /// Implementa el lector recursivo de tipos. `allow_dynamic` se activa al
    /// entrar en los corchetes de una colección, nunca en una declaración
    /// ordinaria, un parámetro o un retorno.
    fn type_name_inner(
        &mut self,
        allow_void: bool,
        allow_dynamic: bool,
    ) -> Result<TypeName, String> {
        let ty =
            match self.peek() {
                Kind::Ident(name) if name == "buffer_nativo" => TypeName::NativeBuffer,
                Kind::Ident(name) if name == "archivo_nativo" => TypeName::NativeFile,
                Kind::Ident(name) if self.classes.contains_key(name) => TypeName::Object(self.classes[name]),
                Kind::Keyword(K::Entero) => TypeName::Int,
                Kind::Keyword(K::Cadena) => TypeName::Str,
                Kind::Keyword(K::Flotante) => TypeName::Float,
                Kind::Keyword(K::Tupla) => TypeName::Tuple,
                Kind::Keyword(K::Conjunto) => TypeName::Set,
                Kind::Keyword(K::Nulo) => TypeName::Null,
                Kind::Keyword(K::Mixto) if allow_dynamic => TypeName::Dynamic,
                Kind::Keyword(K::Lista) => TypeName::List,
                Kind::Keyword(K::Diccionario) => TypeName::Dict,
                Kind::Keyword(K::Booleano) => TypeName::Bool,
                Kind::Keyword(K::Nada) if allow_void => TypeName::None,
                _ => return Err(self.error(
                    "tipo esperado: cadena, flotante, entero, booleano, lista, diccionario, tupla, conjunto o nulo; mixto solo se admite dentro de lista[mixto] (nada solo en retornos)",
                )),
            };
        self.take();
        if matches!(ty, TypeName::List | TypeName::Dict) && self.eat("[") {
            let element = self.type_name_inner(false, true)?;
            self.expect("]")?;
            return TypeName::collection(ty, element).ok_or_else(|| {
                self.error(
                    "elemento no admitido o anidación excesiva (máximo quince niveles internos)",
                )
            });
        }
        Ok(ty)
    }

    /// Acepta punto y coma en argumentos; conserva comas para proyectos anteriores.
    fn eat_separator(&mut self) -> bool {
        self.eat(";") || self.eat(",")
    }

    /// Exige un separador dentro de una llamada o colección, nunca entre sentencias.
    fn separator(&mut self) -> Result<(), String> {
        if self.eat_separator() {
            Ok(())
        } else {
            Err(self.error("se esperaba ; entre argumentos"))
        }
    }

    /// Lee nombre(parámetros tipados) -> retorno { cuerpo } después de funcion.
    /// El retorno omitido equivale a nada. Rechaza parámetros repetidos o más de seis.
    fn function(&mut self, visibility: Visibility) -> Result<FunctionDef, String> {
        let name = self.identifier()?;
        self.expect("(")?;
        let mut params: Vec<Param> = Vec::new();
        if !self.eat(")") {
            loop {
                let pname = self.identifier()?;
                if params.iter().any(|p| p.name == pname) {
                    return Err(self.error("parámetro duplicado"));
                }
                self.expect(":")?;
                params.push(Param {
                    name: pname,
                    ty: self.type_name(false)?,
                });
                if params.len() > 6 {
                    return Err(self.error("se admiten hasta seis parámetros"));
                }
                if self.eat(")") {
                    break;
                }
                self.separator()?;
            }
        }
        let ret = if self.eat("->") {
            self.type_name(true)?
        } else {
            TypeName::None
        };
        self.in_function = true;
        let body = self.block()?;
        self.in_function = false;
        Ok(FunctionDef {
            name,
            params,
            ret,
            body,
            visibility,
        })
    }

    /// Analiza una declaración ya introducida por `variable`.
    /// La declaración siempre inicializa el nombre: ST no admite valores sin inicializar.
    fn variable_declaration(&mut self, visibility: Visibility) -> Result<Stmt, String> {
        let name = self.identifier()?;
        let annotation = if self.eat(":") {
            Some(self.type_name(false)?)
        } else {
            None
        };
        self.expect("=")?;
        let expr = self.expression(0)?;
        self.expect(";")?;
        Ok(Stmt::Assign {
            name,
            annotation,
            expr,
            binding: BindingKind::Declare,
            visibility,
        })
    }

    /// Lee sentencias entre { y }. La sangría no tiene significado en ST.
    /// Un bloque vacío se representa con Pass para reutilizar las etapas comunes.
    fn block(&mut self) -> Result<Vec<Stmt>, String> {
        self.expect("{")?;
        let mut body = Vec::new();
        while !self.eat("}") {
            if self.peek() == &Kind::Eof {
                return Err(self.error("falta '}' para cerrar el bloque"));
            }
            body.push(self.statement()?);
        }
        if body.is_empty() {
            body.push(Stmt::Pass);
        }
        Ok(body)
    }

    /// Lee una sentencia. Las simples terminan con ; y los bloques con }.
    /// Las declaraciones variable y asignaciones se representan con Stmt::Assign;
    /// variable es por ahora un marcador sintáctico, no una regla de redeclaración.
    fn statement(&mut self) -> Result<Stmt, String> {
        // `usar nombre = fábrica(...) { ... }` es contextual; usar(...) sigue siendo una llamada.
        if matches!(self.peek(), Kind::Ident(word) if word == "usar")
            && matches!(
                self.tokens.get(self.pos + 1).map(|t| &t.kind),
                Some(Kind::Ident(_))
            )
        {
            self.take();
            let name = self.identifier()?;
            self.expect("=")?;
            let value = self.expression(0)?;
            let body = self.block()?;
            return Ok(Stmt::Using { name, value, body });
        }

        if self.eat_keyword(K::Lanzar) {
            let message = self.expression(0)?;
            self.expect(";")?;
            return Ok(Stmt::Throw(message));
        }
        if self.eat_keyword(K::Intentar) {
            let body = self.block()?;
            let catch = if self.eat_keyword(K::Capturar) {
                Some((self.identifier()?, self.block()?))
            } else {
                None
            };
            let has_finally = self.eat_keyword(K::Finalmente);
            let finally = if has_finally {
                self.block()?
            } else {
                Vec::new()
            };
            if catch.is_none() && !has_finally {
                return Err(
                    self.error("intentar requiere capturar nombre { ... } o finalmente { ... }")
                );
            }
            return Ok(Stmt::Try {
                body,
                catch,
                finally,
            });
        }

        if self.eat_keyword(K::Coincidir) {
            let value = self.expression(0)?;
            self.expect("{")?;
            let mut cases = Vec::new();
            let mut otherwise = Vec::new();
            while !self.eat("}") {
                if self.eat_keyword(K::Sino) {
                    otherwise = self.block()?;
                    self.expect("}")?;
                    break;
                }
                if !self.eat_keyword(K::Caso) {
                    return Err(self.error("se esperaba caso, sino o }"));
                }
                let pattern = self.expression(0)?;
                // Solo patrones literales: no se ejecutan llamadas ni se crean capturas.
                let literal = matches!(
                    pattern,
                    Expr::Int(_) | Expr::Float(_) | Expr::Bool(_) | Expr::Str(_) | Expr::Null
                ) || matches!(&pattern,Expr::Unary{op:UnaryOp::Neg|UnaryOp::Pos,expr} if matches!(**expr,Expr::Int(_)|Expr::Float(_)));
                if !literal {
                    return Err(self.error("caso requiere un literal escalar"));
                }
                cases.push((pattern, self.block()?));
            }
            return Ok(Stmt::Match {
                value,
                cases,
                otherwise,
            });
        }

        if self.eat_keyword(K::Si) {
            return self.condition();
        }
        if self.eat_keyword(K::Mientras) {
            let cond = self.expression(0)?;
            return Ok(Stmt::While {
                cond,
                body: self.block()?,
            });
        }
        if self.eat_keyword(K::Para) {
            return self.for_range();
        }
        if self.eat_keyword(K::Variable) {
            return self.variable_declaration(Visibility::Private);
        }
        if self.eat_keyword(K::Mostrar) {
            self.expect("(")?;
            let arg = self.expression(0)?;
            self.expect(")")?;
            self.expect(";")?;
            return Ok(Stmt::Expr(Expr::Call {
                name: "print".into(),
                args: vec![arg],
            }));
        }
        if self.eat_keyword(K::Retornar) {
            if !self.in_function {
                return Err(self.error("retornar solo se permite dentro de una función"));
            }
            let expr = if self.eat(";") {
                None
            } else {
                let expr = self.expression(0)?;
                self.expect(";")?;
                Some(expr)
            };
            return Ok(Stmt::Return(expr));
        }
        for (keyword, stmt) in [
            (K::Romper, Stmt::Break),
            (K::Continuar, Stmt::Continue),
            (K::Pasar, Stmt::Pass),
        ] {
            if self.eat_keyword(keyword) {
                self.expect(";")?;
                return Ok(stmt);
            }
        }
        let left = self.expression(0)?;
        let assignment = match self.peek() {
            Kind::Symbol(s) if ["=", "+=", "-=", "*=", "/=", "%="].contains(&s.as_str()) => {
                Some(s.clone())
            }
            _ => None,
        };
        let stmt = if let Some(op) = assignment {
            if let Expr::Index { collection, key } = left {
                if op != "=" {
                    return Err(self.error("usa = para modificar un elemento de colección"));
                }
                self.take();
                let value = self.expression(0)?;
                self.expect(";")?;
                return Ok(Stmt::SetIndex {
                    collection: *collection,
                    key: *key,
                    value,
                });
            }
            if let Expr::Member { object, name } = left {
                if op != "=" {
                    return Err(self.error("usa = para modificar un campo"));
                }
                self.take();
                let value = self.expression(0)?;
                self.expect(";")?;
                return Ok(Stmt::SetMember {
                    object: *object,
                    name,
                    value,
                });
            }
            let Expr::Var(name) = left else {
                return Err(self.error("solo se puede asignar a una variable o elemento"));
            };
            self.take();
            let expr = self.expression(0)?;
            if op == "=" {
                Stmt::Assign {
                    name,
                    annotation: None,
                    expr,
                    binding: BindingKind::Reassign,
                    visibility: Visibility::Private,
                }
            } else {
                let op = match op.as_str() {
                    "+=" => AugOp::Add,
                    "-=" => AugOp::Sub,
                    "*=" => AugOp::Mul,
                    "/=" => AugOp::Div,
                    _ => AugOp::Mod,
                };
                Stmt::AugAssign { name, op, expr }
            }
        } else {
            Stmt::Expr(left)
        };
        self.expect(";")?;
        Ok(stmt)
    }

    /// Lee si condición { ... }, seguido opcionalmente de sino_si, sino si o sino.
    /// Agrupa las ramas en un único Stmt::If como espera lower.
    fn condition(&mut self) -> Result<Stmt, String> {
        let mut branches = Vec::new();
        let mut else_body = Vec::new();
        loop {
            let cond = self.expression(0)?;
            branches.push((cond, self.block()?));
            if self.eat_keyword(K::SinoSi) {
                continue;
            }
            if !self.eat_keyword(K::Sino) {
                break;
            }
            if !self.eat_keyword(K::Si) {
                else_body = self.block()?;
                break;
            }
        }
        Ok(Stmt::If {
            branches,
            else_body,
        })
    }

    /// Lee para nombre en rango(inicio, límite) { ... }; un argumento implica inicio 0.
    /// Genera ForRange y por tanto comparte las limitaciones actuales de lower.
    fn for_range(&mut self) -> Result<Stmt, String> {
        let var = self.identifier()?;
        if !self.eat_keyword(K::En) {
            return Err(self.error("se esperaba la palabra de pertenencia 'en'"));
        }
        if !self.eat_keyword(K::Rango) {
            return Err(self.error("se esperaba rango"));
        }
        self.expect("(")?;
        let first = self.expression(0)?;
        let (start, stop) = if self.eat_separator() {
            (first, self.expression(0)?)
        } else {
            (Expr::Int(0), first)
        };
        self.expect(")")?;
        Ok(Stmt::ForRange {
            var,
            start,
            stop,
            body: self.block()?,
        })
    }

    /// Tabla de operadores y precedencia: valores mayores se agrupan antes.
    /// Para añadir un operador binario, modifica aquí y, si es nuevo, AST/semantic/lower.
    fn binary(&self) -> Option<(BinOp, u8)> {
        Some(match self.peek() {
            Kind::Keyword(K::O) => (BinOp::Or, 1),
            Kind::Keyword(K::Y) => (BinOp::And, 2),
            Kind::Keyword(K::Es) => (BinOp::Is, 3),
            Kind::Symbol(s) => match s.as_str() {
                "==" => (BinOp::Eq, 3),
                "!=" => (BinOp::Ne, 3),
                "<" => (BinOp::Lt, 3),
                "<=" => (BinOp::Le, 3),
                ">" => (BinOp::Gt, 3),
                ">=" => (BinOp::Ge, 3),
                "+" => (BinOp::Add, 4),
                "-" => (BinOp::Sub, 4),
                "*" => (BinOp::Mul, 5),
                "/" => (BinOp::Div, 5),
                "%" => (BinOp::Mod, 5),
                _ => return None,
            },
            _ => return None,
        })
    }

    /// Analizador por precedencia mínima. Consume operandos y agrupa operadores
    /// izquierdos usando prec + 1 para el operando derecho. no se aplica después de
    /// comparaciones y antes de y/o. Rechaza comparaciones encadenadas sin paréntesis.
    fn expression(&mut self, min: u8) -> Result<Expr, String> {
        let mut left = if self.eat_keyword(K::No) {
            Expr::Unary {
                op: UnaryOp::Not,
                expr: Box::new(self.expression(3)?),
            }
        } else if self.eat("-") {
            Expr::Unary {
                op: UnaryOp::Neg,
                expr: Box::new(self.expression(6)?),
            }
        } else if self.eat("+") {
            Expr::Unary {
                op: UnaryOp::Pos,
                expr: Box::new(self.expression(6)?),
            }
        } else {
            self.primary()?
        };
        let mut comparison = false;
        while let Some((op, prec)) = self.binary() {
            if prec < min {
                break;
            }
            if prec == 3 && comparison {
                return Err(self.error("usa paréntesis o 'y' entre comparaciones"));
            }
            comparison |= prec == 3;
            self.take();
            let right = self.expression(prec + 1)?;
            left = Expr::Binary {
                op,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    /// Lee literales, variables, llamadas de usuario y agrupaciones con paréntesis.
    /// mostrar se reserva a sentencias. Las llamadas conservan el orden de argumentos.
    fn primary(&mut self) -> Result<Expr, String> {
        let mut expr = self.atom()?;
        loop {
            if self.eat("[") {
                let key = self.expression(0)?;
                self.expect("]")?;
                expr = Expr::Index {
                    collection: Box::new(expr),
                    key: Box::new(key),
                };
            } else if self.eat(".") {
                // El nombre del método usa el mismo Keyword que la función incorporada.
                // Así los alias de KEYWORDS también funcionan después del punto.
                if !self.eat_keyword(K::Agregar) {
                    let member = self.identifier()?;
                    if !self.eat("(") {
                        expr = Expr::Member {
                            object: Box::new(expr),
                            name: member,
                        };
                        continue;
                    }
                    let mut args = Vec::new();
                    if !self.eat(")") {
                        loop {
                            args.push(self.expression(0)?);
                            if self.eat(")") {
                                break;
                            }
                            self.separator()?;
                        }
                    }
                    expr = Expr::MethodCall {
                        object: Box::new(expr),
                        name: member,
                        args,
                    };
                    continue;
                }
                self.expect("(")?;
                let value = self.expression(0)?;
                self.expect(")")?;
                expr = Expr::Append {
                    collection: Box::new(expr),
                    value: Box::new(value),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    /// Expande una plantilla literal con {} y escapes {{/}} en concatenaciones nativas.
    /// Cada argumento se evalúa una vez y en orden; no interpreta código dentro del texto.
    fn interpolation(&mut self) -> Result<Expr, String> {
        self.expect("(")?;
        let Kind::Str(template) = self.take() else {
            return Err(self.error("interpolar requiere una plantilla literal"));
        };
        let mut args = Vec::new();
        while self.eat_separator() {
            args.push(self.expression(0)?);
        }
        self.expect(")")?;
        let mut chars = template.chars().peekable();
        let mut literal = String::new();
        let mut parts = Vec::new();
        let mut args = args.into_iter();
        while let Some(c) = chars.next() {
            if c == '{' && chars.peek() == Some(&'{') {
                chars.next();
                literal.push('{');
            } else if c == '}' && chars.peek() == Some(&'}') {
                chars.next();
                literal.push('}');
            } else if c == '{' && chars.peek() == Some(&'}') {
                chars.next();
                parts.push(Expr::Str(std::mem::take(&mut literal)));
                let arg = args
                    .next()
                    .ok_or_else(|| self.error("faltan argumentos para los marcadores {}"))?;
                parts.push(Expr::TextOp {
                    name: "cadena".into(),
                    args: vec![arg],
                });
            } else if c == '{' || c == '}' {
                return Err(self.error("usa {} como marcador y {{ o }} para llaves literales"));
            } else {
                literal.push(c);
            }
        }
        if args.next().is_some() {
            return Err(self.error("sobran argumentos de interpolación"));
        }
        parts.push(Expr::Str(literal));
        Ok(parts
            .into_iter()
            .fold(Expr::Str(String::new()), |left, right| Expr::Binary {
                op: BinOp::Add,
                left: Box::new(left),
                right: Box::new(right),
            }))
    }

    /// Lee un átomo o literal de colección. Las claves del diccionario son texto
    /// literal; las funciones incorporadas se representan mediante nodos AST propios.
    fn atom(&mut self) -> Result<Expr, String> {
        match self.peek().clone() {
            Kind::Keyword(K::Yo) => {
                if !self.in_method {
                    return Err(self.error("yo solo se permite dentro de métodos"));
                }
                self.take();
                Ok(Expr::Var("yo".into()))
            }
            Kind::Keyword(K::Nuevo) => {
                self.take();
                let first = self.identifier()?;
                let member = if self.eat(".") {
                    Some(self.identifier()?)
                } else {
                    None
                };
                self.expect("(")?;
                let mut args = Vec::new();
                if !self.eat(")") {
                    loop {
                        args.push(self.expression(0)?);
                        if self.eat(")") {
                            break;
                        }
                        self.separator()?;
                    }
                }
                let call = if let Some(name) = member {
                    Expr::MethodCall {
                        object: Box::new(Expr::Var(first)),
                        name,
                        args,
                    }
                } else {
                    Expr::Call { name: first, args }
                };
                Ok(Expr::New(Box::new(call)))
            }
            Kind::Symbol(s) if s == "[" => {
                self.take();
                let mut values = Vec::new();
                if !self.eat("]") {
                    loop {
                        values.push(self.expression(0)?);
                        if self.eat("]") {
                            break;
                        }
                        self.separator()?;
                    }
                }
                Ok(Expr::List(values))
            }
            Kind::Symbol(s) if s == "{" => {
                self.take();
                let mut values = Vec::new();
                if !self.eat("}") {
                    loop {
                        let key = self.expression(0)?;
                        self.expect(":")?;
                        values.push((key, self.expression(0)?));
                        if self.eat("}") {
                            break;
                        }
                        self.separator()?;
                    }
                }
                Ok(Expr::Dict(values))
            }
            Kind::Keyword(K::Interpolar) => {
                self.take();
                self.interpolation()
            }
            Kind::Keyword(K::Entrada) | Kind::Keyword(K::Abrir) | Kind::Keyword(K::Escribir) => {
                let (name, count) = match self.take() {
                    Kind::Keyword(K::Entrada) => ("entrada", 0),
                    Kind::Keyword(K::Abrir) => ("abrir", 1),
                    _ => ("escribir", 2),
                };
                self.expect("(")?;
                let mut args = Vec::new();
                if name == "entrada" && self.peek() != &Kind::Symbol(")".into()) {
                    args.push(self.expression(0)?);
                }
                for i in 0..count {
                    if i != 0 {
                        self.separator()?;
                    }
                    args.push(self.expression(0)?);
                }
                self.expect(")")?;
                Ok(Expr::TextOp {
                    name: name.into(),
                    args,
                })
            }
            Kind::Keyword(K::Buscar)
            | Kind::Keyword(K::Reemplazar)
            | Kind::Keyword(K::Cadena)
            | Kind::Keyword(K::Entero)
            | Kind::Keyword(K::Flotante)
            | Kind::Keyword(K::Booleano) => {
                let (name, count) = match self.take() {
                    Kind::Keyword(K::Buscar) => ("buscar", 2),
                    Kind::Keyword(K::Reemplazar) => ("reemplazar", 3),
                    Kind::Keyword(K::Entero) => ("entero", 1),
                    Kind::Keyword(K::Flotante) => ("flotante", 1),
                    Kind::Keyword(K::Booleano) => ("booleano", 1),
                    _ => ("cadena", 1),
                };
                self.expect("(")?;
                let mut args = Vec::new();
                for i in 0..count {
                    if i != 0 {
                        self.separator()?;
                    }
                    args.push(self.expression(0)?);
                }
                self.expect(")")?;
                Ok(Expr::TextOp {
                    name: name.into(),
                    args,
                })
            }
            Kind::Keyword(K::Nulo) => {
                self.take();
                Ok(Expr::Null)
            }
            Kind::Keyword(K::Tupla) | Kind::Keyword(K::Conjunto) => {
                // Los constructores evalúan sus elementos de izquierda a derecha.
                let tuple = self.take() == Kind::Keyword(K::Tupla);
                self.expect("(")?;
                let mut values = Vec::new();
                if !self.eat(")") {
                    loop {
                        values.push(self.expression(0)?);
                        if self.eat(")") {
                            break;
                        }
                        self.separator()?;
                        if self.eat(")") {
                            break;
                        }
                    }
                }
                Ok(if tuple {
                    Expr::Tuple(values)
                } else {
                    Expr::Set(values)
                })
            }
            Kind::Keyword(K::Agregar) | Kind::Keyword(K::Longitud) => {
                let append = self.take() == Kind::Keyword(K::Agregar);
                self.expect("(")?;
                let collection = Box::new(self.expression(0)?);
                let expression = if append {
                    self.separator()?;
                    let value = Box::new(self.expression(0)?);
                    Expr::Append { collection, value }
                } else {
                    Expr::Length(collection)
                };
                self.expect(")")?;
                Ok(expression)
            }
            Kind::Float(n) => {
                self.take();
                Ok(Expr::Float(n))
            }
            Kind::Int(n) => {
                self.take();
                Ok(Expr::Int(n))
            }
            Kind::Str(s) => {
                self.take();
                Ok(Expr::Str(s))
            }
            Kind::Keyword(K::Verdadero) => {
                self.take();
                Ok(Expr::Bool(true))
            }
            Kind::Keyword(K::Falso) => {
                self.take();
                Ok(Expr::Bool(false))
            }
            Kind::Ident(_) => {
                let name = self.identifier()?;
                if !self.eat("(") {
                    return Ok(Expr::Var(name));
                }
                let mut args = Vec::new();
                if !self.eat(")") {
                    loop {
                        args.push(self.expression(0)?);
                        if self.eat(")") {
                            break;
                        }
                        self.separator()?;
                    }
                }
                Ok(Expr::Call { name, args })
            }
            Kind::Symbol(s) if s == "(" => {
                self.take();
                let expr = self.expression(0)?;
                self.expect(")")?;
                Ok(expr)
            }
            _ => Err(self.error("se esperaba una expresión")),
        }
    }
}
