//! Análisis semántico: AST → tablas de símbolos y tipos.
//! driver llama a analyze antes de lower. Se registran firmas, variables y retornos
//! para comprobar operaciones y orientar la generación de IR. No ejecuta el programa
//! La verificación posterior de IR comprueba inicialización y retorno en rutas alcanzables.
use crate::ast::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
/// Firma sin cuerpo: params contiene tipos en orden y ret el tipo de retorno.
/// Permite comprobar llamadas antes de analizar los cuerpos de las funciones.
pub struct FnSig {
    pub params: Vec<TypeName>,
    pub ret: TypeName,
    /// El resultado es propiedad nueva y puede ser adquirido por `usar`.
    pub fresh_resource: bool,
}

#[derive(Debug, Clone, Default)]
/// Tablas para lower: functions guarda firmas, main_vars tipos del bloque principal
/// y fn_vars una tabla de variables por cada función.
pub struct SemanticInfo {
    pub functions: HashMap<String, FnSig>,
    pub main_vars: HashMap<String, TypeName>,
    pub fn_vars: HashMap<String, HashMap<String, TypeName>>,
}

/// Registra primero las firmas de `program` para permitir recursión y referencias
/// a otras funciones. Rechaza funciones duplicadas y comprueba el bloque principal
/// y cada cuerpo con sus parámetros iniciales. Devuelve SemanticInfo para lower.
/// La verificación posterior de IR rechaza rutas alcanzables sin retorno.
pub fn analyze(program: &Program) -> Result<SemanticInfo, String> {
    let mut info = SemanticInfo::default();
    for f in &program.functions {
        if f.name == "__main__" {
            return Err("nombre reservado del compilador: __main__".into());
        }
        if f.name == "main" && !f.params.is_empty() {
            return Err("la función main no admite parámetros".into());
        }
        if info.functions.contains_key(&f.name) {
            return Err(format!("función duplicada: {}", f.name));
        }
        info.functions.insert(
            f.name.clone(),
            FnSig {
                params: f.params.iter().map(|p| p.ty).collect(),
                ret: f.ret,
                fresh_resource: matches!(f.body.as_slice(),
                    [Stmt::Return(Some(Expr::TextOp { name, .. }))]
                    if crate::st::stdlib::native(name).is_some_and(|n| n.result.is_resource())),
            },
        );
    }
    // Propaga el efecto de propiedad por envoltorios simples. No basta que una
    // función devuelva el tipo correcto: debe llamar a una fábrica ya comprobada.
    loop {
        let mut changed = false;
        for f in &program.functions {
            if !f.ret.is_resource() || info.functions[&f.name].fresh_resource {
                continue;
            }
            let fresh = matches!(f.body.as_slice(),
                [Stmt::Return(Some(Expr::Call { name, .. }))]
                if info.functions.get(name).is_some_and(|sig| sig.fresh_resource));
            if fresh {
                info.functions.get_mut(&f.name).unwrap().fresh_resource = true;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for f in &program.functions {
        if f.ret.is_resource() && !info.functions[&f.name].fresh_resource {
            return Err(format!(
                "{}: no puede retornar un préstamo; el recurso debe proceder de una fábrica comprobada",
                f.name
            ));
        }
    }
    // El bloque principal se comprueba como retorno Int para el estado de salida.
    let mut main = HashMap::new();
    check_block(&program.body, &mut main, &info.functions, TypeName::Int, 0)?;
    info.main_vars = main;
    for f in &program.functions {
        let mut vars = HashMap::new();
        for p in &f.params {
            vars.insert(p.name.clone(), p.ty);
        }
        check_block(&f.body, &mut vars, &info.functions, f.ret, 0)?;
        info.fn_vars.insert(f.name.clone(), vars);
    }
    Ok(info)
}

/// Comprueba `stmts` en orden y actualiza la tabla mutable `vars`.
/// `fns` contiene firmas, `ret_ty` el retorno esperado y `loop_depth` permite
/// validar break/continue. Revisa anotaciones, tipos estables y condiciones.
/// If usa copias que descarta; while incorpora nombres nuevos de su copia y
/// for modifica la tabla recibida. Estas reglas no garantizan inicialización
/// en todas las rutas. Devuelve () o el primer error encontrado.
fn check_block(
    stmts: &[Stmt],
    vars: &mut HashMap<String, TypeName>,
    fns: &HashMap<String, FnSig>,
    ret_ty: TypeName,
    loop_depth: usize,
) -> Result<(), String> {
    for s in stmts {
        match s {
            Stmt::Using { name, value, body } => {
                if vars.contains_key(name) {
                    return Err(format!("usar {name}: el nombre ya está definido"));
                }
                let ty = infer_expr(value, vars, fns)?;
                let fresh = matches!(value, Expr::Call { name, .. }
                    if fns.get(name).is_some_and(|sig| sig.fresh_resource));
                if !ty.is_resource() || !fresh {
                    return Err("usar requiere una llamada que cree un archivo_nativo o buffer_nativo nuevo".into());
                }
                let mut scoped = vars.clone();
                scoped.insert(name.clone(), ty);
                check_block(body, &mut scoped, fns, ret_ty, loop_depth)?;
            }

            Stmt::SetMember {
                object,
                name,
                value,
            } => {
                let call = crate::st::objects::member_call(
                    object,
                    "set",
                    name,
                    &[value.clone()],
                    vars,
                    fns,
                )?;
                infer_expr(&call, vars, fns)?;
            }
            Stmt::SetRecordSlot { .. } => {} // Solo generado por setters con parámetros tipados.

            Stmt::Throw(message) => {
                if infer_expr(message, vars, fns)? != TypeName::Str {
                    return Err("lanzar requiere un mensaje de tipo cadena".into());
                }
            }
            Stmt::Try {
                body,
                catch,
                finally,
            } => {
                // Se registran locales de cada camino con tipos estables. La IR
                // rechaza usos que no estén inicializados al entrar al capturador.
                check_block(body, vars, fns, ret_ty, loop_depth)?;
                if let Some((name, handler)) = catch {
                    if vars.get(name).is_some_and(|ty| *ty != TypeName::Str) {
                        return Err(format!(
                            "capturar {name}: el nombre ya tiene un tipo distinto de cadena"
                        ));
                    }
                    vars.insert(name.clone(), TypeName::Str);
                    check_block(handler, vars, fns, ret_ty, loop_depth)?;
                }
                check_block(finally, vars, fns, ret_ty, loop_depth)?;
            }

            Stmt::Match {
                value,
                cases,
                otherwise,
            } => {
                let ty = infer_expr(value, vars, fns)?;
                if ty.is_collection() || ElementType::from_type(ty).is_none() {
                    return Err("coincidir requiere un valor escalar".into());
                }
                for (pattern, body) in cases {
                    if infer_expr(pattern, vars, fns)? != ty {
                        return Err("el patrón debe tener el tipo del valor comparado".into());
                    }
                    check_block(body, &mut vars.clone(), fns, ret_ty, loop_depth)?;
                }
                check_block(otherwise, &mut vars.clone(), fns, ret_ty, loop_depth)?;
            }

            Stmt::SetIndex {
                collection,
                key,
                value,
            } => {
                if infer_expr(collection, vars, fns)? == TypeName::Tuple
                    || infer_expr(collection, vars, fns)? == TypeName::Str
                {
                    return Err(
                        "Las tuplas y cadenas son inmutables; no se pueden asignar sus elementos"
                            .into(),
                    );
                }
                check_index(collection, key, vars, fns)?;
                let collection_ty = infer_expr(collection, vars, fns)?;
                let value_ty = infer_expr(value, vars, fns)?;
                if collection_ty.element() == Some(TypeName::Dynamic) {
                    if !dynamic_list_value(value_ty) {
                        return Err("lista[mixto] no admite recursos ni valores sin retorno".into());
                    }
                } else if Some(value_ty) != collection_ty.element() {
                    return Err(
                        "El valor no coincide con el tipo de elemento de la colección".into(),
                    );
                }
            }
            Stmt::Assign {
                name,
                annotation,
                expr,
                binding,
                ..
            } => {
                let previous = vars.get(name).copied();
                match binding {
                    BindingKind::Declare if previous.is_some() => {
                        return Err(format!(
                            "variable ya declarada: {name}; usa {name} = ... para reasignarla"
                        ));
                    }
                    BindingKind::Reassign if previous.is_none() => {
                        return Err(format!(
                            "variable no declarada: {name}; créala primero con variable {name} = ..."
                        ));
                    }
                    _ => {}
                }
                let mut ty = infer_expr(expr, vars, fns)?;
                if ty.is_resource() || previous.is_some_and(TypeName::is_resource) {
                    return Err(format!("{name}: los recursos no se copian ni reasignan; adquiere propiedad con usar"));
                }
                // Un literal vacío toma el tipo de una anotación o variable ya existente.
                if matches!(expr,Expr::List(v) if v.is_empty())
                    || matches!(expr,Expr::Dict(v) if v.is_empty())
                {
                    if let Some(want) = annotation.as_ref().or_else(|| vars.get(name)) {
                        if ty.is_list() && want.is_list() || ty.is_dict() && want.is_dict() {
                            ty = *want;
                        }
                    }
                }
                if let Some(a) = annotation {
                    if *a == TypeName::Dynamic {
                        return Err("mixto solo puede usarse como elemento de lista[mixto]".into());
                    }
                    if !type_compatible(*a, ty) {
                        return Err(format!(
                            "tipo incompatible para {name}: anotado {}, recibido {}",
                            a.as_str(),
                            ty.as_str()
                        ));
                    }
                    // La anotación conserva la interfaz solicitada. En particular,
                    // lista[mixto] hace que cada lectura produzca un valor etiquetado.
                    ty = *a;
                }
                if let Some(old) = previous {
                    if !type_compatible(old, ty) {
                        return Err(format!("la variable {name} cambia de tipo de {} a {}; se requieren tipos estables",old.as_str(),ty.as_str()));
                    }
                    ty = old;
                }
                vars.insert(name.clone(), ty);
            }
            Stmt::AugAssign { name, op, expr } => {
                let old = *vars
                    .get(name)
                    .ok_or_else(|| format!("variable no definida: {name}"))?;
                let rhs = infer_expr(expr, vars, fns)?;
                if old == TypeName::Str
                    && matches!(op, AugOp::Add)
                    && matches!(
                        rhs,
                        TypeName::Str
                            | TypeName::Int
                            | TypeName::Float
                            | TypeName::Bool
                            | TypeName::Null
                    )
                {
                    continue;
                }
                if old != rhs
                    || !(old == TypeName::Int
                        || old == TypeName::Float && !matches!(op, AugOp::Mod)
                        || old == TypeName::Str && matches!(op, AugOp::Add))
                {
                    return Err(format!("asignación aumentada incompatible para {name}"));
                }
            }
            Stmt::Expr(e) => {
                if infer_expr(e, vars, fns)?.is_resource() {
                    return Err(
                        "un recurso adquirido debe quedar dentro de usar; no se puede descartar"
                            .into(),
                    );
                }
                if let Expr::Call { name, args } = e {
                    if name == "print" {
                        if args.len() != 1 {
                            return Err("la impresión admite exactamente un argumento".into());
                        }
                        let ty = infer_expr(&args[0], vars, fns)?;
                        if ty.is_resource() {
                            return Err("no se pueden convertir recursos; usa su API tipada".into());
                        }
                        if ty.is_resource()
                            || ty.is_collection()
                            || matches!(ty, TypeName::Object(_))
                        {
                            return Err("mostrar requiere un entero, booleano o texto literal; accede a un elemento de la colección".into());
                        }
                    } else {
                        let _ = infer_expr(e, vars, fns)?;
                    }
                } else {
                    let _ = infer_expr(e, vars, fns)?;
                }
            }
            Stmt::Return(e) => match (ret_ty, e) {
                (TypeName::None, None) => {}
                (TypeName::None, Some(_)) => {
                    return Err("una función sin retorno no puede devolver un valor".into())
                }
                (_, None) => {
                    if ret_ty != TypeName::Unknown {
                        return Err(format!(
                            "una función de retorno {} debe devolver un valor",
                            ret_ty.as_str()
                        ));
                    }
                }
                (_, Some(x)) => {
                    let t = infer_expr(x, vars, fns)?;
                    if t.is_resource() && !matches!(x, Expr::Call { .. } | Expr::TextOp { .. }) {
                        return Err(
                            "no se puede retornar un recurso prestado o propiedad de usar".into(),
                        );
                    }
                    if ret_ty != TypeName::Unknown && !type_compatible(ret_ty, t) {
                        return Err(format!(
                            "retorno incompatible: se esperaba {}, se recibió {}",
                            ret_ty.as_str(),
                            t.as_str()
                        ));
                    }
                }
            },
            // Las copias de variables de las ramas se descartan tras comprobarlas.
            Stmt::If {
                branches,
                else_body,
            } => {
                for (c, b) in branches {
                    let ct = infer_expr(c, vars, fns)?;
                    if ct != TypeName::Bool && ct != TypeName::Int && ct != TypeName::Unknown {
                        return Err("la condición debe ser booleana o entera".into());
                    }
                    let mut child = vars.clone();
                    check_block(b, &mut child, fns, ret_ty, loop_depth)?;
                }
                let mut child = vars.clone();
                check_block(else_body, &mut child, fns, ret_ty, loop_depth)?;
            }
            // Se incorporan nombres del cuerpo aunque el bucle pudiera no ejecutarse.
            Stmt::While { cond, body } => {
                let ct = infer_expr(cond, vars, fns)?;
                if ct != TypeName::Bool && ct != TypeName::Int && ct != TypeName::Unknown {
                    return Err("la condición del bucle debe ser booleana o entera".into());
                }
                let mut child = vars.clone();
                check_block(body, &mut child, fns, ret_ty, loop_depth + 1)?;
                for (k, v) in child {
                    if !vars.contains_key(&k) {
                        vars.insert(k, v);
                    }
                }
            }
            Stmt::ForRange {
                var,
                start,
                stop,
                body,
            } => {
                if infer_expr(start, vars, fns)? != TypeName::Int
                    || infer_expr(stop, vars, fns)? != TypeName::Int
                {
                    return Err("los límites del rango deben ser enteros".into());
                }
                if vars.get(var).is_some_and(|ty| *ty != TypeName::Int) {
                    return Err(format!("la variable de rango {var} debe ser entera"));
                }
                vars.insert(var.clone(), TypeName::Int);
                check_block(body, vars, fns, ret_ty, loop_depth + 1)?;
            }
            Stmt::Break | Stmt::Continue => {
                if loop_depth == 0 {
                    return Err("romper/continuar usados fuera de un bucle".into());
                }
            }
            Stmt::Pass => {}
        }
    }
    Ok(())
}

/// Deduce el TypeName de `e` consultando `vars` y `fns`, sin calcular su valor.
/// Comprueba recursivamente operandos, tipos y cantidad de argumentos.
/// La aritmética admite enteros o pares de flotantes; + también concatena cadenas.
/// La lógica admite bool/int y devuelve bool. No hay conversión implícita entre tipos.
/// print se trata como None; su aridad se valida en check_block.
pub fn infer_expr(
    e: &Expr,
    vars: &HashMap<String, TypeName>,
    fns: &HashMap<String, FnSig>,
) -> Result<TypeName, String> {
    match e {
        Expr::Member { object, name } => {
            let call = crate::st::objects::member_call(object, "get", name, &[], vars, fns)?;
            infer_expr(&call, vars, fns)
        }
        Expr::MethodCall { object, name, args } => {
            let call = crate::st::objects::member_call(object, "method", name, args, vars, fns)?;
            infer_expr(&call, vars, fns)
        }
        Expr::New(call) => {
            let ty = infer_expr(call, vars, fns)?;
            if !matches!(ty, TypeName::Object(_)) {
                return Err("nuevo requiere un constructor que devuelva un objeto".into());
            }
            Ok(ty)
        }
        Expr::Record { id, .. } => Ok(TypeName::Object(*id)),
        Expr::RecordSlot { ty, .. } => Ok(*ty),

        Expr::TextOp { name, args } => {
            // Conversiones explícitas: ninguna permite reinterpretar una referencia como número.
            if matches!(name.as_str(), "entero" | "flotante" | "booleano") {
                if args.len() != 1 {
                    return Err(format!("{name} espera exactamente un argumento"));
                }
                let ty = infer_expr(&args[0], vars, fns)?;
                if ty.is_resource() {
                    return Err("no se pueden convertir recursos; usa su API tipada".into());
                }
                let valid = if name == "booleano" {
                    !matches!(ty, TypeName::None | TypeName::Unknown)
                } else {
                    matches!(
                        ty,
                        TypeName::Int | TypeName::Float | TypeName::Bool | TypeName::Str
                    )
                };
                if !valid && ty != TypeName::Dynamic {
                    return Err(format!("no se puede convertir {} a {name}", ty.st_name()));
                }
                return Ok(match name.as_str() {
                    "entero" => TypeName::Int,
                    "flotante" => TypeName::Float,
                    _ => TypeName::Bool,
                });
            }
            // Consultar el tipo comprueba la expresión: no permite variables inexistentes.
            if name == "tipo" {
                if let Some(arg) = args.first() {
                    if infer_expr(arg, vars, fns)?.is_resource() && !matches!(arg, Expr::Var(_)) {
                        return Err(
                            "tipo no puede adquirir un recurso temporal; usa primero usar".into(),
                        );
                    }
                }
                if args.len() != 1 {
                    return Err("tipo espera exactamente un argumento".into());
                }
                let ty = infer_expr(&args[0], vars, fns)?;
                if matches!(ty, TypeName::None | TypeName::Unknown) {
                    return Err("tipo requiere una expresión con valor; una función sin retorno no produce un valor".into());
                }
                return Ok(TypeName::Str);
            }
            // Las primitivas estándar tienen firmas explícitas, nunca tipos inferidos por nombre.
            if let Some(native) = crate::st::stdlib::native(name) {
                if args.len() != native.args.len() {
                    return Err(format!("aridad incorrecta: {name}"));
                }
                for (arg, expected) in args.iter().zip(native.args) {
                    if infer_expr(arg, vars, fns)? != *expected {
                        return Err(format!(
                            "tipo incorrecto en {name}: se esperaba {expected:?}"
                        ));
                    }
                }
                return Ok(native.result);
            }
            if name == "entrada" {
                if args.len() > 1
                    || args
                        .iter()
                        .any(|arg| infer_expr(arg, vars, fns) != Ok(TypeName::Str))
                {
                    return Err(
                        "entrada admite ningún argumento o un mensaje de tipo cadena".into(),
                    );
                }
                return Ok(TypeName::Str);
            }
            if name == "cadena" {
                if args.len() != 1 {
                    return Err("cadena espera exactamente un argumento".into());
                }
                if !matches!(
                    infer_expr(&args[0], vars, fns)?,
                    TypeName::Str
                        | TypeName::Int
                        | TypeName::Float
                        | TypeName::Bool
                        | TypeName::Null
                        | TypeName::Dynamic
                ) {
                    return Err("cadena requiere un valor escalar".into());
                }
                return Ok(TypeName::Str);
            }
            for arg in args {
                if infer_expr(arg, vars, fns)? != TypeName::Str {
                    return Err(format!("{name} requiere argumentos cadena"));
                }
            }
            Ok(if name == "escribir" {
                TypeName::None
            } else if name == "buscar" {
                TypeName::Int
            } else {
                TypeName::Str
            })
        }

        Expr::Null => Ok(TypeName::Null),
        Expr::Tuple(values) | Expr::Set(values) => {
            for value in values {
                if infer_expr(value, vars, fns)? != TypeName::Int {
                    return Err("Tuplas y conjuntos solo admiten enteros".into());
                }
            }
            Ok(if matches!(e, Expr::Tuple(_)) {
                TypeName::Tuple
            } else {
                TypeName::Set
            })
        }
        Expr::List(values) => {
            let mut element = None;
            let mut heterogeneous = false;
            for value in values {
                let ty = infer_expr(value, vars, fns)?;
                if !dynamic_list_value(ty) {
                    return Err("la lista no admite recursos ni valores sin retorno".into());
                }
                if element.is_some_and(|old| old != ty) {
                    heterogeneous = true;
                }
                element = Some(ty);
            }
            if heterogeneous {
                Ok(TypeName::ListOf(ElementType::Dynamic))
            } else {
                let element = element.unwrap_or(TypeName::Int);
                if let Some(list) = TypeName::collection(TypeName::List, element) {
                    Ok(list)
                } else if matches!(element, TypeName::Object(_)) {
                    // Los objetos son referencias válidas, pero su identidad nominal
                    // no cabe en el árbol compacto de colecciones homogéneas.
                    Ok(TypeName::ListOf(ElementType::Dynamic))
                } else {
                    Err("la lista supera la profundidad máxima de quince niveles internos".into())
                }
            }
        }
        Expr::Dict(values) => {
            let mut element = None;
            for (key, value) in values {
                if infer_expr(key, vars, fns)? != TypeName::Str {
                    return Err("La clave del diccionario debe ser cadena".into());
                }
                let ty = infer_expr(value, vars, fns)?;
                if ElementType::from_type(ty).is_none() || element.is_some_and(|old| old != ty) {
                    return Err("el diccionario requiere valores del mismo tipo y profundidad representable".into());
                }
                element = Some(ty);
            }
            Ok(TypeName::collection(TypeName::Dict, element.unwrap_or(TypeName::Int)).unwrap())
        }
        Expr::Index { collection, key } => {
            check_index(collection, key, vars, fns)?;
            Ok(if infer_expr(collection, vars, fns)? == TypeName::Str {
                TypeName::Str
            } else {
                infer_expr(collection, vars, fns)?.element().unwrap()
            })
        }
        Expr::Length(collection) => {
            let ty = infer_expr(collection, vars, fns)?;
            if !ty.is_collection() && ty != TypeName::Str {
                return Err(
                    "longitud requiere una cadena, lista, diccionario, tupla o conjunto".into(),
                );
            }
            Ok(TypeName::Int)
        }
        Expr::Append { collection, value } => {
            let ty = infer_expr(collection, vars, fns)?;
            let value_ty = infer_expr(value, vars, fns)?;
            let valid = if ty.element() == Some(TypeName::Dynamic) {
                ty.is_list() && dynamic_list_value(value_ty)
            } else {
                (ty.is_list() || ty == TypeName::Set) && ty.element() == Some(value_ty)
            };
            if !valid {
                return Err(
                    "agregar requiere una lista/conjunto y un valor del tipo de sus elementos"
                        .into(),
                );
            }
            Ok(TypeName::None)
        }
        Expr::Float(_) => Ok(TypeName::Float),
        Expr::Int(_) => Ok(TypeName::Int),
        Expr::Bool(_) => Ok(TypeName::Bool),
        Expr::Str(_) => Ok(TypeName::Str),
        Expr::Var(n) => vars
            .get(n)
            .copied()
            .ok_or_else(|| format!("variable no definida: {n}")),
        Expr::Unary { op, expr } => {
            let t = infer_expr(expr, vars, fns)?;
            match op {
                UnaryOp::Neg | UnaryOp::Pos
                    if matches!(t, TypeName::Int | TypeName::Float | TypeName::Unknown) =>
                {
                    Ok(t)
                }
                UnaryOp::Not
                    if t == TypeName::Bool || t == TypeName::Int || t == TypeName::Unknown =>
                {
                    Ok(TypeName::Bool)
                }
                _ => Err(format!("operación unaria no válida para {}", t.as_str())),
            }
        }
        Expr::Binary { op, left, right } => {
            let l = infer_expr(left, vars, fns)?;
            if l.is_resource() || infer_expr(right, vars, fns)?.is_resource() {
                return Err("los recursos no admiten operadores; usa su API tipada".into());
            }
            let r = infer_expr(right, vars, fns)?;
            match op {
                BinOp::Add if l == TypeName::Unknown || r == TypeName::Unknown => {
                    Ok(TypeName::Unknown)
                }
                BinOp::Add
                    if (l == TypeName::Str || r == TypeName::Str)
                        && matches!(
                            l,
                            TypeName::Str
                                | TypeName::Int
                                | TypeName::Float
                                | TypeName::Bool
                                | TypeName::Null
                        )
                        && matches!(
                            r,
                            TypeName::Str
                                | TypeName::Int
                                | TypeName::Float
                                | TypeName::Bool
                                | TypeName::Null
                        ) =>
                {
                    Ok(TypeName::Str)
                }
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div
                    if l == TypeName::Float && r == TypeName::Float =>
                {
                    Ok(TypeName::Float)
                }
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod
                    if l == TypeName::Unknown || r == TypeName::Unknown =>
                {
                    Ok(TypeName::Unknown)
                }
                BinOp::Is => {
                    let null_case = l == TypeName::Null || r == TypeName::Null;
                    if (l == TypeName::Unknown || r == TypeName::Unknown)
                        || (l == r
                            && (l == TypeName::Null
                                || l.is_collection()
                                || matches!(l, TypeName::Object(_))))
                        || null_case
                    {
                        Ok(TypeName::Bool)
                    } else {
                        Err("es requiere referencias del mismo tipo o dos valores nulos; para números usa ==".into())
                    }
                }
                BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                    if l == TypeName::Int && r == TypeName::Int {
                        Ok(TypeName::Int)
                    } else {
                        Err("la operación requiere enteros o dos flotantes compatibles (sin mezcla implícita)".into())
                    }
                }
                BinOp::Eq | BinOp::Ne => {
                    if l == TypeName::Unknown || r == TypeName::Unknown || l == r {
                        Ok(TypeName::Bool)
                    } else {
                        Err("los operandos de comparación deben tener el mismo tipo".into())
                    }
                }
                BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                    if l == TypeName::Unknown
                        || r == TypeName::Unknown
                        || (l == r && matches!(l, TypeName::Int | TypeName::Float))
                    {
                        Ok(TypeName::Bool)
                    } else {
                        Err(
                            "las comparaciones de orden requieren dos enteros o dos flotantes"
                                .into(),
                        )
                    }
                }
                BinOp::And | BinOp::Or => {
                    if (l == TypeName::Unknown || r == TypeName::Unknown)
                        || ((l == TypeName::Bool || l == TypeName::Int)
                            && (r == TypeName::Bool || r == TypeName::Int))
                    {
                        Ok(TypeName::Bool)
                    } else {
                        Err("los operadores lógicos requieren booleanos o enteros".into())
                    }
                }
            }
        }
        Expr::Call { name, args } => {
            if name == "print" {
                return Ok(TypeName::None);
            }
            // Los intrínsecos validan su representación antes de emitir accesos a memoria.
            if !fns.contains_key(name)
                && matches!(name.as_str(), "len" | "abs" | "sqrt" | "__contains__")
            {
                let expected = if name == "__contains__" { 2 } else { 1 };
                if args.len() != expected {
                    return Err(format!("{name}: número de argumentos inválido"));
                }
                let first = infer_expr(&args[0], vars, fns)?;
                return match name.as_str() {
                    "len" if first.is_collection() || first == TypeName::Str => Ok(TypeName::Int),
                    "abs" if matches!(first, TypeName::Int | TypeName::Float) => Ok(first),
                    "sqrt" if matches!(first, TypeName::Int | TypeName::Float) => {
                        Ok(TypeName::Float)
                    }
                    "__contains__" => {
                        let collection = infer_expr(&args[1], vars, fns)?;
                        if (collection.element() == Some(TypeName::Dynamic)
                            && collection.is_list()
                            && dynamic_list_value(first))
                            || ((collection.is_list() && collection.element() == Some(first))
                                || (matches!(collection, TypeName::Tuple | TypeName::Set)
                                    && first == TypeName::Int))
                            || (collection.is_dict() && first == TypeName::Str)
                        {
                            Ok(TypeName::Bool)
                        } else {
                            Err("pertenencia: se requiere colección de enteros o clave de diccionario".into())
                        }
                    }
                    _ => Err(format!("{name}: tipo de argumento no admitido")),
                };
            }
            let sig = fns
                .get(name)
                .cloned()
                .or_else(|| builtin_sig(name))
                .ok_or_else(|| format!("función desconocida: {name}"))?;
            if args.len() != sig.params.len() {
                return Err(format!(
                    "la función {name} espera {} argumentos, recibió {}",
                    sig.params.len(),
                    args.len()
                ));
            }
            for (i, (a, want)) in args.iter().zip(&sig.params).enumerate() {
                let got = infer_expr(a, vars, fns)?;
                if got.is_resource() && !matches!(a, Expr::Var(_)) {
                    return Err("el préstamo requiere una variable de recurso viva; adquiere primero con usar".into());
                }
                if *want != TypeName::Unknown && !type_compatible(*want, got) {
                    return Err(format!(
                        "argumento {} de {name}: se esperaba {}, se recibió {}",
                        i + 1,
                        want.as_str(),
                        got.as_str()
                    ));
                }
            }
            Ok(sig.ret)
        }
    }
}

/// Decide si un valor cabe en una lista heterogénea sin perder propiedad ni ABI.
/// Los recursos nativos deben seguir gobernados por `usar`, y `nada` no es un valor.
fn dynamic_list_value(ty: TypeName) -> bool {
    !matches!(ty, TypeName::None | TypeName::Unknown) && !ty.is_resource()
}

/// Compatibilidad direccional entre el tipo declarado y el valor recibido.
/// Una lista mixta acepta listas tipadas completas como valores, pero una lista
/// homogénea nunca acepta una lista mixta que podría contener otro tipo.
fn type_compatible(expected: TypeName, received: TypeName) -> bool {
    expected == received
        || (expected == TypeName::ListOf(ElementType::Dynamic) && received.is_list())
}

fn builtin_sig(name: &str) -> Option<FnSig> {
    let params = match name {
        "sqrt" | "abs" => vec![TypeName::Unknown],
        "len" => vec![TypeName::Unknown],
        "__contains__" => vec![TypeName::Unknown, TypeName::Unknown],
        _ => return None,
    };
    let ret = match name {
        "sqrt" | "abs" => TypeName::Float,
        "len" => TypeName::Int,
        "__contains__" => TypeName::Bool,
        _ => return None,
    };
    Some(FnSig {
        params,
        ret,
        fresh_resource: false,
    })
}

/// Comprueba el tipo de acceso antes de emitir instrucciones de memoria.
/// Las secuencias usan índices enteros; los diccionarios aceptan expresiones cadena.
fn check_index(
    collection: &Expr,
    key: &Expr,
    vars: &HashMap<String, TypeName>,
    fns: &HashMap<String, FnSig>,
) -> Result<(), String> {
    let ty = infer_expr(collection, vars, fns)?;
    let index = infer_expr(key, vars, fns)?;
    if (ty.is_list() || ty == TypeName::Tuple || ty == TypeName::Str) && index == TypeName::Int {
        return Ok(());
    }
    if ty.is_dict() && index == TypeName::Str {
        return Ok(());
    }
    Err("acceso no válido: secuencias requieren entero y diccionarios requieren cadena".into())
}

/// Reconstruye los locales de un ámbito ya validado, sin filtrar su propietario al exterior.
pub fn resource_scope_vars(
    body: &[Stmt],
    vars: &HashMap<String, TypeName>,
    fns: &HashMap<String, FnSig>,
    ret: TypeName,
    depth: usize,
) -> Result<HashMap<String, TypeName>, String> {
    let mut scoped = vars.clone();
    check_block(body, &mut scoped, fns, ret, depth)?;
    Ok(scoped)
}
