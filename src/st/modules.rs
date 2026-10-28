//! Carga de módulos y resolución de funciones importadas.
//! Las importaciones antiguas conservan el espacio global cuando todo el grafo es antiguo.
//! Al usar alias o selección, cada módulo resuelve sus propias funciones e importaciones.
use crate::ast::*;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

/// Declaración analizada; names contiene pares (exportado, nombre local).
#[derive(Debug, Clone)]
pub struct Import {
    pub path: String,
    pub alias: Option<String>,
    pub names: Vec<(String, String)>,
}
/// Un nodo contiene su AST y los identificadores de sus dependencias ya cargadas.
struct Module {
    path: PathBuf,
    program: Program,
    standard: bool,
    imports: Vec<(Import, usize)>,
}

/// Símbolos que un módulo permite importar. Funciones y variables se mantienen
/// separadas porque una llamada y una lectura producen nodos AST diferentes.
#[derive(Default)]
struct Exports {
    functions: HashMap<String, String>,
    variables: HashMap<String, String>,
}

/// Carga dependencias estándar en orden topológico sin buscar código externo.
/// Conserva los mismos alias y detecta ciclos igual que las importaciones locales.
fn visit_standard(
    name: &str,
    active: &mut Vec<PathBuf>,
    done: &mut HashMap<PathBuf, usize>,
    out: &mut Vec<Module>,
) -> Result<usize, String> {
    let key = PathBuf::from(format!("<estandar>/{name}"));
    if active.contains(&key) {
        return Err(format!("Importación estándar circular: {name}"));
    }
    if let Some(id) = done.get(&key) {
        return Ok(*id);
    }
    active.push(key.clone());
    let (program, imports) =
        super::stdlib::load(name).map_err(|e| format!("módulo estándar {name}: {e}"))?;
    let mut dependencies = Vec::new();
    for mut import in imports {
        if !super::stdlib::contains(&import.path) {
            return Err(format!(
                "El módulo estándar {name} requiere otro módulo estándar: {}",
                import.path
            ));
        }
        let id = visit_standard(&import.path, active, done, out)?;
        if import.alias.is_none() && import.names.is_empty() {
            import.alias = Some(import.path.clone());
        }
        dependencies.push((import, id));
    }
    active.pop();
    let id = out.len();
    out.push(Module {
        path: key.clone(),
        program,
        standard: true,
        imports: dependencies,
    });
    done.insert(key, id);
    Ok(id)
}

/// Carga una vez cada ruta canónica, resuelve funciones y combina inicializadores en orden.
pub fn load(path: &Path) -> Result<Program, String> {
    let mut modules = Vec::new();
    visit(path, &mut Vec::new(), &mut HashMap::new(), &mut modules)?;
    let entry_id = modules.len().saturating_sub(1);
    let mut exports: Vec<Exports> = Vec::new();
    let mut out = Program::default();
    for (id, mut module) in modules.into_iter().enumerate() {
        let result = (|| {
            let mut functions = HashMap::new();
            let mut variables = HashMap::new();
            let mut owned_variables = HashMap::new();
            let local_functions: Vec<_> = module
                .program
                .functions
                .iter()
                .map(|function| {
                    let target = if function.name.starts_with("__obj_") {
                        function.name.clone()
                    } else if id == entry_id && function.name == "main" {
                        "main".into()
                    } else {
                        format!("__module_{id}_{}", function.name)
                    };
                    (function.name.clone(), function.visibility, target)
                })
                .collect();
            let mut declared_variables = Vec::new();
            collect_declared_variables(&module.program.body, &mut declared_variables);
            for (name, _, visibility) in &declared_variables {
                let target = format!("__module_{id}_var_{name}");
                bind(&mut variables, name.clone(), target.clone())?;
                bind(&mut owned_variables, name.clone(), target)?;
                // Solo una declaración `publica variable` inmediata del módulo se exporta.
                // Las variables de bloques siguen siendo locales aunque el AST las recorra.
                if *visibility == Visibility::Public
                    && !module.program.body.iter().any(|statement| {
                        matches!(statement, Stmt::Assign { name: candidate, visibility: Visibility::Public, .. } if candidate == name)
                    })
                {
                    return Err(format!("{name}: una variable pública debe declararse en el nivel del módulo"));
                }
            }
            for (name, _, target) in &local_functions {
                bind(&mut functions, name.clone(), target.clone())?;
            }
            for (import, dependency) in &module.imports {
                let available = &exports[*dependency];
                if !import.names.is_empty() {
                    for (name, local) in &import.names {
                        match (available.functions.get(name), available.variables.get(name)) {
                            (Some(_), Some(_)) => {
                                return Err(format!(
                                    "{name} es ambiguo en {}; usa nombres diferentes",
                                    import.path
                                ));
                            }
                            (Some(target), None) => {
                                bind(&mut functions, local.clone(), target.clone())?;
                            }
                            (None, Some(target)) => {
                                bind(&mut variables, local.clone(), target.clone())?;
                            }
                            (None, None) => {
                                return Err(format!(
                                    "el símbolo público {name} no existe en {}",
                                    import.path
                                ));
                            }
                        }
                    }
                } else {
                    for (name, target) in &available.functions {
                        let local = import
                            .alias
                            .as_ref()
                            .map(|alias| format!("{alias}.{name}"))
                            .unwrap_or_else(|| name.clone());
                        bind(&mut functions, local, target.clone())?;
                    }
                    for (name, target) in &available.variables {
                        let local = import
                            .alias
                            .as_ref()
                            .map(|alias| format!("{alias}.{name}"))
                            .unwrap_or_else(|| name.clone());
                        bind(&mut variables, local, target.clone())?;
                    }
                }
            }
            for f in &mut module.program.functions {
                f.name = functions[&f.name].clone();
                // Las variables superiores no se capturan implícitamente: cada función
                // conserva locales y parámetros propios como en el modelo seguro de ST.
                resolve_block(&mut f.body, &functions, &HashMap::new(), &HashMap::new())?;
            }
            resolve_block(
                &mut module.program.body,
                &functions,
                &variables,
                &owned_variables,
            )?;

            let mut public = Exports::default();
            for (name, visibility, target) in local_functions {
                if visibility == Visibility::Public
                    && (!module.standard || !name.starts_with("aux_"))
                    && !name.starts_with("__obj_")
                {
                    public.functions.insert(name, target);
                }
            }
            for statement in &module.program.body {
                if let Stmt::Assign {
                    name,
                    visibility: Visibility::Public,
                    binding: BindingKind::Declare,
                    ..
                } = statement
                {
                    // resolve_block ya cambió `name` al símbolo interno. Se recupera
                    // la escritura pública original mediante el mapa inverso.
                    if let Some((source, _)) =
                        owned_variables.iter().find(|(_, target)| *target == name)
                    {
                        public.variables.insert(source.clone(), name.clone());
                    }
                }
            }
            Ok(public)
        })();
        exports.push(result.map_err(|e: String| format!("{}: {e}", module.path.display()))?);
        out.functions.append(&mut module.program.functions);
        out.body.append(&mut module.program.body);
    }
    Ok(out)
}

/// Resuelve tipo(valor) después de reunir funciones: las funciones de usuario conservan prioridad.
/// El marcador permite dejar otros nombres sin resolver para el análisis semántico habitual.
pub fn resolve_type_queries(program: &mut Program) -> Result<(), String> {
    let mut bindings: HashMap<String, String> = program
        .functions
        .iter()
        .map(|f| (f.name.clone(), f.name.clone()))
        .collect();
    bindings.insert("__allow_unknown".into(), String::new());
    for function in &mut program.functions {
        resolve_block(
            &mut function.body,
            &bindings,
            &HashMap::new(),
            &HashMap::new(),
        )?;
    }
    resolve_block(
        &mut program.body,
        &bindings,
        &HashMap::new(),
        &HashMap::new(),
    )
}

/// Inserta un nombre; un mismo símbolo puede llegar por un diamante sin duplicarse.
fn bind(
    bindings: &mut HashMap<String, String>,
    name: String,
    target: String,
) -> Result<(), String> {
    if let Some(previous) = bindings.get(&name) {
        if previous != &target {
            return Err(format!("nombre importado o función ambiguos: {name}"));
        }
    }
    bindings.insert(name, target);
    Ok(())
}

/// DFS con estados activo/completo: detecta ciclos antes de deduplicar la inicialización.
/// Resuelve @nombre dentro de bibliotecas/nombre/src/lib.st del proyecto más cercano.
/// La ruta canónica impide escapar mediante enlaces simbólicos. No descarga ni ejecuta instaladores.
fn library_path(importer: &Path, name: &str) -> Result<PathBuf, String> {
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err("Nombre de biblioteca inválido: usa @nombre con letras, números, _ o -".into());
    }
    let root = importer
        .ancestors()
        .skip(1)
        .find(|p| p.join("PyForge.toml").is_file())
        .ok_or("Las bibliotecas @nombre requieren un proyecto con PyForge.toml")?;
    let directory = root.join("bibliotecas");
    let base = fs::canonicalize(&directory).map_err(|e| format!("Bibliotecas locales: {e}"))?;
    if !base.starts_with(root) {
        return Err("La carpeta bibliotecas debe permanecer dentro del proyecto".into());
    }
    let entry = fs::canonicalize(base.join(name).join("src/lib.st"))
        .map_err(|e| format!("Biblioteca @{name} no disponible: {e}"))?;
    if !entry.starts_with(&base) {
        return Err("La biblioteca no puede escapar del directorio bibliotecas".into());
    }
    Ok(entry)
}

fn visit(
    path: &Path,
    active: &mut Vec<PathBuf>,
    done: &mut HashMap<PathBuf, usize>,
    out: &mut Vec<Module>,
) -> Result<usize, String> {
    if path.extension().and_then(|s| s.to_str()) != Some("st") {
        return Err(format!(
            "Solo se pueden importar módulos .st: {}",
            path.display()
        ));
    }
    let path = fs::canonicalize(path)
        .map_err(|e| format!("No se puede abrir el módulo {}: {e}", path.display()))?;
    if active.contains(&path) {
        return Err(format!(
            "Importación circular: {}",
            active
                .iter()
                .chain(std::iter::once(&path))
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(" -> ")
        ));
    }
    if let Some(id) = done.get(&path) {
        return Ok(*id);
    }
    let source = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let (program, imports) =
        super::parser::parse_module(&source).map_err(|e| format!("{}: {e}", path.display()))?;
    active.push(path.clone());
    let mut dependencies = Vec::new();
    for mut import in imports {
        if super::stdlib::contains(&import.path) {
            let id = visit_standard(&import.path, active, done, out)?;
            if import.alias.is_none() && import.names.is_empty() {
                import.alias = Some(import.path.clone());
            }
            dependencies.push((import, id));
            continue;
        }
        if Path::new(&import.path).is_absolute() {
            return Err(format!(
                "{}: las importaciones deben usar rutas relativas",
                path.display()
            ));
        }
        let resolved = if let Some(name) = import.path.strip_prefix('@') {
            library_path(&path, name)?
        } else {
            path.parent().unwrap().join(&import.path)
        };
        let id = visit(&resolved, active, done, out)?;
        dependencies.push((import, id));
    }
    active.pop();
    let id = out.len();
    done.insert(path.clone(), id);
    out.push(Module {
        path,
        program,
        standard: false,
        imports: dependencies,
    });
    Ok(id)
}

/// Reúne todos los nombres que pertenecen al bloque principal del módulo.
/// Se renombran para que dos archivos puedan usar el mismo local sin compartirlo.
fn collect_declared_variables(
    statements: &[Stmt],
    output: &mut Vec<(String, BindingKind, Visibility)>,
) {
    for statement in statements {
        match statement {
            Stmt::Assign {
                name,
                binding: BindingKind::Declare,
                visibility,
                ..
            } => output.push((name.clone(), BindingKind::Declare, *visibility)),
            Stmt::Using { name, body, .. } => {
                output.push((name.clone(), BindingKind::Declare, Visibility::Private));
                collect_declared_variables(body, output);
            }
            Stmt::Try {
                body,
                catch,
                finally,
            } => {
                collect_declared_variables(body, output);
                if let Some((name, handler)) = catch {
                    output.push((name.clone(), BindingKind::Declare, Visibility::Private));
                    collect_declared_variables(handler, output);
                }
                collect_declared_variables(finally, output);
            }
            Stmt::If {
                branches,
                else_body,
            } => {
                for (_, body) in branches {
                    collect_declared_variables(body, output);
                }
                collect_declared_variables(else_body, output);
            }
            Stmt::Match {
                cases, otherwise, ..
            } => {
                for (_, body) in cases {
                    collect_declared_variables(body, output);
                }
                collect_declared_variables(otherwise, output);
            }
            Stmt::While { body, .. } => collect_declared_variables(body, output),
            Stmt::ForRange { var, body, .. } => {
                output.push((var.clone(), BindingKind::Declare, Visibility::Private));
                collect_declared_variables(body, output);
            }
            _ => {}
        }
    }
}

/// Resuelve llamadas y lecturas de variables importadas. Los mapas separados
/// evitan confundir `modulo.funcion()` con `modulo.variable`.
fn resolve_expr(
    expr: &mut Expr,
    functions: &HashMap<String, String>,
    variables: &HashMap<String, String>,
) -> Result<(), String> {
    // Función contextual ST: no reserva el identificador ni interfiere con json.tipo().
    if let Expr::Call { name, args } = expr {
        if name == "tipo" && !functions.contains_key(name) {
            for arg in args.iter_mut() {
                resolve_expr(arg, functions, variables)?;
            }
            *expr = Expr::TextOp {
                name: "tipo".into(),
                args: std::mem::take(args),
            };
            return Ok(());
        }
    }
    // Un alias de módulo se distingue antes de resolver métodos de objetos.
    if let Expr::MethodCall { object, name, args } = expr {
        if let Expr::Var(alias) = &**object {
            if let Some(target) = functions.get(&format!("{alias}.{name}")) {
                let mut args = args.clone();
                for arg in &mut args {
                    resolve_expr(arg, functions, variables)?;
                }
                *expr = Expr::Call {
                    name: target.clone(),
                    args,
                };
                return Ok(());
            }
        }
    }
    // Una lectura `alias.nombre` puede referirse a una variable pública del módulo.
    if let Expr::Member { object, name } = expr {
        if let Expr::Var(alias) = &**object {
            if let Some(target) = variables.get(&format!("{alias}.{name}")) {
                *expr = Expr::Var(target.clone());
                return Ok(());
            }
        }
    }
    match expr {
        Expr::Member { object, .. } | Expr::RecordSlot { object, .. } | Expr::New(object) => {
            resolve_expr(object, functions, variables)?
        }
        Expr::MethodCall { object, args, .. } => {
            resolve_expr(object, functions, variables)?;
            for arg in args {
                resolve_expr(arg, functions, variables)?;
            }
        }
        Expr::Record { values, .. } => {
            for value in values {
                resolve_expr(value, functions, variables)?;
            }
        }

        Expr::Call { name, args } => {
            if name != "print" {
                if let Some(target) = functions.get(name) {
                    *name = target.clone();
                } else if !matches!(name.as_str(), "len" | "abs" | "sqrt" | "__contains__")
                    && !functions.contains_key("__allow_unknown")
                {
                    return Err(format!("función no visible en el módulo: {name}"));
                }
            }
            for arg in args {
                resolve_expr(arg, functions, variables)?;
            }
        }
        Expr::TextOp { args, .. } | Expr::List(args) | Expr::Tuple(args) | Expr::Set(args) => {
            for arg in args {
                resolve_expr(arg, functions, variables)?;
            }
        }
        Expr::Length(expr) | Expr::Unary { expr, .. } => resolve_expr(expr, functions, variables)?,
        Expr::Append { collection, value } => {
            resolve_expr(collection, functions, variables)?;
            resolve_expr(value, functions, variables)?;
        }
        Expr::Index { collection, key } => {
            resolve_expr(collection, functions, variables)?;
            resolve_expr(key, functions, variables)?;
        }
        Expr::Binary { left, right, .. } => {
            resolve_expr(left, functions, variables)?;
            resolve_expr(right, functions, variables)?;
        }
        Expr::Dict(entries) => {
            for (key, value) in entries {
                resolve_expr(key, functions, variables)?;
                resolve_expr(value, functions, variables)?;
            }
        }
        Expr::Null | Expr::Float(_) | Expr::Int(_) | Expr::Bool(_) | Expr::Str(_) => {}
        Expr::Var(name) => {
            if let Some(target) = variables.get(name) {
                *name = target.clone();
            }
        }
    }
    Ok(())
}

/// Recorre todos los cuerpos, incluidas selecciones, para evitar llamadas sin resolver.
fn resolve_block(
    stmts: &mut [Stmt],
    functions: &HashMap<String, String>,
    variables: &HashMap<String, String>,
    owned_variables: &HashMap<String, String>,
) -> Result<(), String> {
    for stmt in stmts {
        match stmt {
            Stmt::Using { name, value, body } => {
                resolve_expr(value, functions, variables)?;
                if let Some(target) = owned_variables.get(name) {
                    *name = target.clone();
                }
                resolve_block(body, functions, variables, owned_variables)?;
            }
            Stmt::SetMember {
                object: Expr::Var(alias),
                name,
                ..
            } if variables.contains_key(&format!("{alias}.{name}")) => {
                return Err(format!(
                    "la variable pública {alias}.{name} es de solo lectura fuera de su módulo"
                ));
            }
            Stmt::SetMember { object, value, .. } | Stmt::SetRecordSlot { object, value, .. } => {
                resolve_expr(object, functions, variables)?;
                resolve_expr(value, functions, variables)?;
            }

            Stmt::Try {
                body,
                catch,
                finally,
            } => {
                resolve_block(body, functions, variables, owned_variables)?;
                if let Some((name, handler)) = catch {
                    if let Some(target) = owned_variables.get(name) {
                        *name = target.clone();
                    }
                    resolve_block(handler, functions, variables, owned_variables)?;
                }
                resolve_block(finally, functions, variables, owned_variables)?;
            }
            Stmt::Throw(expr) => resolve_expr(expr, functions, variables)?,
            Stmt::Assign { name, expr, .. } => {
                resolve_expr(expr, functions, variables)?;
                if let Some(target) = owned_variables.get(name) {
                    *name = target.clone();
                }
            }
            Stmt::AugAssign { name, expr, .. } => {
                resolve_expr(expr, functions, variables)?;
                if let Some(target) = owned_variables.get(name) {
                    *name = target.clone();
                }
            }
            Stmt::Expr(expr) | Stmt::Return(Some(expr)) => {
                resolve_expr(expr, functions, variables)?
            }
            Stmt::SetIndex {
                collection,
                key,
                value,
            } => {
                resolve_expr(collection, functions, variables)?;
                resolve_expr(key, functions, variables)?;
                resolve_expr(value, functions, variables)?;
            }
            Stmt::If {
                branches,
                else_body,
            } => {
                for (expr, body) in branches {
                    resolve_expr(expr, functions, variables)?;
                    resolve_block(body, functions, variables, owned_variables)?;
                }
                resolve_block(else_body, functions, variables, owned_variables)?;
            }
            Stmt::Match {
                value,
                cases,
                otherwise,
            } => {
                resolve_expr(value, functions, variables)?;
                for (expr, body) in cases {
                    resolve_expr(expr, functions, variables)?;
                    resolve_block(body, functions, variables, owned_variables)?;
                }
                resolve_block(otherwise, functions, variables, owned_variables)?;
            }
            Stmt::While { cond, body } => {
                resolve_expr(cond, functions, variables)?;
                resolve_block(body, functions, variables, owned_variables)?;
            }
            Stmt::ForRange {
                var,
                start,
                stop,
                body,
            } => {
                resolve_expr(start, functions, variables)?;
                resolve_expr(stop, functions, variables)?;
                if let Some(target) = owned_variables.get(var) {
                    *var = target.clone();
                }
                resolve_block(body, functions, variables, owned_variables)?;
            }
            Stmt::Return(None) | Stmt::Break | Stmt::Continue | Stmt::Pass => {}
        }
    }
    Ok(())
}
