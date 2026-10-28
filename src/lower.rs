//! Conversión del AST comprobado a la representación intermedia de pila.
//! Recibe SemanticInfo de semantic y produce ProgramIr para optimizer y codegen.
//! Una expresión normalmente apila un valor; una sentencia lo almacena, imprime
//! o descarta. Las condiciones y bucles se expresan mediante etiquetas y saltos.
use crate::ast::*;
use crate::ir::*;
use crate::semantic::{infer_expr, SemanticInfo};
use std::collections::HashMap;

/// Convierte `program` y su análisis `sem` en ProgramIr.
/// Genera funciones con locales separados de parámetros y un retorno implícito
/// para funciones None. Envuelve las sentencias superiores en __main__ con
/// retorno cero. Las tablas de sem deben corresponder al AST: se consultan con unwrap.
pub fn lower(program: &Program, sem: &SemanticInfo) -> Result<ProgramIr, String> {
    let mut l = Lowerer {
        out: ProgramIr::default(),
        label: 0,
        temporaries: Vec::new(),
        loops: Vec::new(),
        cleanups: Vec::new(),
        fns: &sem.functions,
        return_type: TypeName::Int,
    };
    for f in &program.functions {
        // El análisis semántico debe haber creado una tabla para cada función.
        l.return_type = f.ret;
        let vars = sem.fn_vars.get(&f.name).unwrap();
        let mut code = Vec::new();
        l.block(&f.body, vars, &mut code)?;
        if f.ret == TypeName::None {
            code.push(Instr::ReturnVoid);
        }
        // Los parámetros ya figuran aparte; evita tratarlos como locales adicionales.
        let mut locals: Vec<String> = vars
            .keys()
            .filter(|n| !f.params.iter().any(|p| &p.name == *n))
            .cloned()
            .collect();
        locals.append(&mut l.temporaries);
        l.out.functions.push(FunctionIr {
            name: f.name.clone(),
            params: f.params.iter().map(|p| p.name.clone()).collect(),
            locals,
            return_type: f.ret,
            code,
        });
    }
    l.return_type = TypeName::Int;
    // Envuelve el código superior en __main__, que devuelve cero al terminar.
    // Si existe `main`, se ejecuta primero y su valor de retorno se descarta para
    // mantener compatibilidad con el resto del programa y con el flujo actual sin romper.
    let mut code = Vec::new();
    if program.functions.iter().any(|f| f.name == "main") {
        code.push(Instr::Call {
            name: "main".into(),
            argc: 0,
        });
        code.push(Instr::Pop);
    }
    l.block(&program.body, &sem.main_vars, &mut code)?;
    code.push(Instr::PushI64(0));
    code.push(Instr::ReturnValue);
    l.out.main = Some(FunctionIr {
        name: "__main__".into(),
        params: vec![],
        locals: sem
            .main_vars
            .keys()
            .cloned()
            .chain(l.temporaries.drain(..))
            .collect(),
        return_type: TypeName::Int,
        code,
    });
    Ok(l.out)
}

/// Estado de conversión: out acumula IR y cadenas, label numera etiquetas únicas,
/// loops apila pares (destino_continue, destino_break) y fns toma prestadas firmas.
/// La vida a expresa que esas firmas deben existir mientras se usa este estado.
struct Lowerer<'a> {
    out: ProgramIr,
    label: usize,
    /// Locales internos generados para evaluar selecciones una sola vez.
    temporaries: Vec<String>,
    loops: Vec<(String, String, usize)>,
    /// Registros activos con su limpieza; se retiran de dentro hacia fuera.
    cleanups: Vec<(String, Vec<Stmt>, Vec<(String, String, usize)>)>,
    fns: &'a HashMap<String, crate::semantic::FnSig>,
    return_type: TypeName,
}
impl<'a> Lowerer<'a> {
    /// Reserva un local imposible de escribir desde ST (el punto no es identificador).
    fn temporary(&mut self) -> String {
        let name = self.new_label(".temporal");
        self.temporaries.push(name.clone());
        name
    }

    /// Retira los manejadores abandonados por return/break/continue y ejecuta
    /// sus limpiezas. Se excluye la limpieza actual para que no se llame a sí misma.
    fn cleanup_to(
        &mut self,
        depth: usize,
        vars: &HashMap<String, TypeName>,
        code: &mut Vec<Instr>,
    ) -> Result<(), String> {
        let saved = self.cleanups.clone();
        let saved_loops = self.loops.clone();
        while self.cleanups.len() > depth {
            let (record, finally, loops) = self.cleanups.pop().unwrap();
            self.loops = loops;
            code.push(Instr::TryEnd { record });
            self.block(&finally, vars, code)?;
        }
        self.cleanups = saved;
        self.loops = saved_loops;
        Ok(())
    }

    /// Un finally envuelve el try/catch interior con otro manejador. Así también
    /// limpia cuando el capturador lanza de nuevo; el mensaje se guarda antes de
    /// ejecutar código que podría capturar otra excepción y cambiar el mensaje global.
    fn try_stmt(
        &mut self,
        body: &[Stmt],
        catch: &Option<(String, Vec<Stmt>)>,
        finally: &[Stmt],
        vars: &HashMap<String, TypeName>,
        code: &mut Vec<Instr>,
    ) -> Result<(), String> {
        let record = self.temporary();
        let handler = self.new_label("exception");
        let end = self.new_label("try_end");
        code.push(Instr::TryBegin {
            record: record.clone(),
            catch: handler.clone(),
        });
        self.cleanups
            .push((record.clone(), finally.to_vec(), self.loops.clone()));
        if !finally.is_empty() && catch.is_some() {
            self.try_stmt(body, catch, &[], vars, code)?;
        } else {
            self.block(body, vars, code)?;
        }
        self.cleanups.pop();
        code.push(Instr::TryEnd { record });
        self.block(finally, vars, code)?;
        code.push(Instr::Jump(end.clone()));
        code.push(Instr::Label(handler));
        if finally.is_empty() {
            if let Some((name, handler_body)) = catch {
                code.push(Instr::ExceptionValue);
                code.push(Instr::Store(name.clone()));
                self.block(handler_body, vars, code)?;
            } else {
                code.push(Instr::ExceptionValue);
                code.push(Instr::Throw);
            }
        } else {
            let message = self.temporary();
            code.push(Instr::ExceptionValue);
            code.push(Instr::Store(message.clone()));
            self.block(finally, vars, code)?;
            code.push(Instr::Load(message));
            code.push(Instr::Throw);
        }
        code.push(Instr::Label(end));
        Ok(())
    }

    /// Convierte el valor ya apilado a descriptor de texto; respeta el formato de mostrar.
    fn text_conversion(ty: TypeName, code: &mut Vec<Instr>) {
        let op = match ty {
            TypeName::Int => ScalarOp::IntText,
            TypeName::Float => ScalarOp::FloatText,
            TypeName::Bool => ScalarOp::BoolText,
            TypeName::Null => ScalarOp::NullText,
            _ => return,
        };
        code.push(Instr::Scalar(op));
    }

    /// Selecciona la rutina IEEE-754; no utiliza operaciones enteras sobre los bits.
    fn float_op(op: BinOp) -> Result<ScalarOp, String> {
        Ok(match op {
            BinOp::Add => ScalarOp::FloatAdd,
            BinOp::Sub => ScalarOp::FloatSub,
            BinOp::Mul => ScalarOp::FloatMul,
            BinOp::Div => ScalarOp::FloatDiv,
            BinOp::Eq => ScalarOp::FloatEq,
            BinOp::Ne => ScalarOp::FloatNe,
            BinOp::Lt => ScalarOp::FloatLt,
            BinOp::Le => ScalarOp::FloatLe,
            BinOp::Gt => ScalarOp::FloatGt,
            BinOp::Ge => ScalarOp::FloatGe,
            _ => return Err("operación flotante no admitida".into()),
        })
    }

    /// Emite colección y clave una sola vez en orden; las claves son descriptores de texto.
    fn index(
        &mut self,
        collection: &Expr,
        key: &Expr,
        vars: &HashMap<String, TypeName>,
        code: &mut Vec<Instr>,
    ) -> Result<bool, String> {
        let dict = infer_expr(collection, vars, self.fns)?.is_dict();
        self.expr(collection, vars, code)?;
        self.expr(key, vars, code)?;
        Ok(dict)
    }
    /// Devuelve una etiqueta con prefijo `p` y contador creciente.
    /// Incrementa label compartido por todo el programa para evitar colisiones de saltos.
    fn new_label(&mut self, p: &str) -> String {
        let n = self.label;
        self.label += 1;
        format!("{p}_{n}")
    }
    /// Añade `s` como bytes UTF-8 más un salto de línea a out.strings.
    /// Devuelve su índice para PrintStr. No deduplica: cada llamada añade una entrada.
    fn intern(&mut self, s: &str) -> usize {
        let mut b = s.as_bytes().to_vec();
        b.push(b'\n');
        self.out.strings.push(b);
        self.out.strings.len() - 1
    }
    /// Convierte las sentencias `stmts` en orden usando stmt.
    /// `vars` aporta tipos y `code` acumula instrucciones; propaga el primer error.
    fn block(
        &mut self,
        stmts: &[Stmt],
        vars: &HashMap<String, TypeName>,
        code: &mut Vec<Instr>,
    ) -> Result<(), String> {
        for s in stmts {
            self.stmt(s, vars, code)?;
        }
        Ok(())
    }
    /// Emite en `code` la IR de `s`, usando `vars` para elegir la impresión.
    /// Asignaciones guardan valores; expresiones sueltas los descartan; return termina
    /// la función. If y bucles generan etiquetas y saltos; loops guarda los destinos
    /// de continue/break del bucle interior. Pass no emite código.
    /// El for actual reevalúa stop en cada vuelta y usa la variable como contador,
    /// por lo que no reproduce todos los detalles de range de Python.
    fn stmt(
        &mut self,
        s: &Stmt,
        vars: &HashMap<String, TypeName>,
        code: &mut Vec<Instr>,
    ) -> Result<(), String> {
        match s {
            Stmt::Using { name, value, body } => {
                let ty = infer_expr(value, vars, self.fns)?;
                self.expr(value, vars, code)?;
                code.push(Instr::Store(name.clone()));
                let mut scoped = vars.clone();
                scoped.insert(name.clone(), ty);
                let scoped = crate::semantic::resource_scope_vars(
                    body,
                    &scoped,
                    self.fns,
                    self.return_type,
                    self.loops.len(),
                )?;
                for local in scoped.keys() {
                    if !vars.contains_key(local) && !self.temporaries.contains(local) {
                        self.temporaries.push(local.clone());
                    }
                }
                let close = if ty == TypeName::NativeFile {
                    "aux_cerrar_archivo"
                } else {
                    "aux_cerrar_buffer"
                };
                let cleanup = Stmt::Expr(Expr::TextOp {
                    name: format!("__std_recursos_{close}"),
                    args: vec![Expr::Var(name.clone())],
                });
                self.try_stmt(body, &None, &[cleanup], &scoped, code)?;
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
                    self.fns,
                )?;
                self.expr(&call, vars, code)?;
                code.push(Instr::Pop);
            }
            Stmt::SetRecordSlot {
                object,
                index,
                value,
            } => {
                self.expr(object, vars, code)?;
                code.push(Instr::PushI64(*index as i64));
                self.expr(value, vars, code)?;
                code.push(Instr::PushI64(runtime_tag(infer_expr(
                    value, vars, self.fns,
                )?)?));
                code.push(Instr::Collection(CollectionOp::ListSetTagged));
                code.push(Instr::Pop);
            }

            Stmt::Try {
                body,
                catch,
                finally,
            } => self.try_stmt(body, catch, finally, vars, code)?,
            Stmt::Throw(message) => {
                self.expr(message, vars, code)?;
                code.push(Instr::Throw);
            }

            Stmt::Match {
                value,
                cases,
                otherwise,
            } => {
                let temporary = self.new_label("__match_value");
                self.temporaries.push(temporary.clone());
                self.expr(value, vars, code)?;
                code.push(Instr::Store(temporary.clone()));
                let mut scoped = vars.clone();
                scoped.insert(temporary.clone(), infer_expr(value, vars, self.fns)?);
                let end = self.new_label("match_end");
                for (pattern, body) in cases {
                    let next = self.new_label("match_next");
                    self.expr(
                        &Expr::Binary {
                            op: BinOp::Eq,
                            left: Box::new(Expr::Var(temporary.clone())),
                            right: Box::new(pattern.clone()),
                        },
                        &scoped,
                        code,
                    )?;
                    code.push(Instr::JumpIfFalse(next.clone()));
                    self.block(body, &scoped, code)?;
                    code.push(Instr::Jump(end.clone()));
                    code.push(Instr::Label(next));
                }
                self.block(otherwise, &scoped, code)?;
                code.push(Instr::Label(end));
            }

            Stmt::SetIndex {
                collection,
                key,
                value,
            } => {
                let dict = self.index(collection, key, vars, code)?;
                self.expr(value, vars, code)?;
                let value_type = infer_expr(value, vars, self.fns)?;
                let operation = if dict {
                    CollectionOp::TextDictSet
                } else if value_type == TypeName::Dynamic {
                    CollectionOp::ListSetDynamic
                } else {
                    code.push(Instr::PushI64(runtime_tag(value_type)?));
                    CollectionOp::ListSetTagged
                };
                code.push(Instr::Collection(operation));
                code.push(Instr::Pop);
            }
            Stmt::Assign { name, expr, .. } => {
                self.expr(expr, vars, code)?;
                code.push(Instr::Store(name.clone()));
            }
            // Carga el valor anterior, evalúa el derecho, opera y almacena el resultado.
            Stmt::AugAssign { name, op, expr } => {
                code.push(Instr::Load(name.clone()));
                self.expr(expr, vars, code)?;
                let ty = vars[name];
                if ty == TypeName::Str {
                    Self::text_conversion(infer_expr(expr, vars, self.fns)?, code);
                    code.push(Instr::Scalar(ScalarOp::TextConcat));
                } else if ty == TypeName::Float {
                    code.push(Instr::Scalar(Self::float_op(match op {
                        AugOp::Add => BinOp::Add,
                        AugOp::Sub => BinOp::Sub,
                        AugOp::Mul => BinOp::Mul,
                        AugOp::Div => BinOp::Div,
                        AugOp::Mod => BinOp::Mod,
                    })?));
                } else {
                    code.push(match op {
                        AugOp::Add => Instr::Add,
                        AugOp::Sub => Instr::Sub,
                        AugOp::Mul => Instr::Mul,
                        AugOp::Div => Instr::Div,
                        AugOp::Mod => Instr::Mod,
                    });
                }
                code.push(Instr::Store(name.clone()));
            }
            Stmt::Expr(Expr::Call { name, args }) if name == "print" => {
                let a = &args[0];
                let ty = infer_expr(a, vars, self.fns)?;
                if ty == TypeName::Null {
                    self.expr(a, vars, code)?;
                    code.push(Instr::Pop);
                    let id = self.intern("nulo");
                    code.push(Instr::PrintStr(id));
                } else if matches!(ty, TypeName::Str | TypeName::Float | TypeName::Dynamic) {
                    self.expr(a, vars, code)?;
                    code.push(Instr::Scalar(match ty {
                        TypeName::Str => ScalarOp::TextPrint,
                        TypeName::Float => ScalarOp::FloatPrint,
                        TypeName::Dynamic => ScalarOp::DynamicPrint,
                        _ => unreachable!(),
                    }));
                    code.push(Instr::Pop);
                } else {
                    self.expr(a, vars, code)?;
                    code.push(if ty == TypeName::Bool {
                        Instr::PrintBool
                    } else {
                        Instr::PrintInt
                    });
                }
            }
            Stmt::Expr(e) => {
                self.expr(e, vars, code)?;
                code.push(Instr::Pop);
            }
            Stmt::Return(Some(e)) => {
                self.expr(e, vars, code)?;
                if !self.cleanups.is_empty() {
                    let value = self.temporary();
                    code.push(Instr::Store(value.clone()));
                    self.cleanup_to(0, vars, code)?;
                    code.push(Instr::Load(value));
                }
                code.push(Instr::ReturnValue);
            }
            Stmt::Return(None) => {
                self.cleanup_to(0, vars, code)?;
                code.push(Instr::ReturnVoid);
            }
            // Cada rama falsa salta a la siguiente; una rama ejecutada salta al final común.
            Stmt::If {
                branches,
                else_body,
            } => {
                let end = self.new_label("if_end");
                for (cond, body) in branches {
                    let next = self.new_label("if_next");
                    self.expr(cond, vars, code)?;
                    code.push(Instr::JumpIfFalse(next.clone()));
                    self.block(body, vars, code)?;
                    code.push(Instr::Jump(end.clone()));
                    code.push(Instr::Label(next));
                }
                self.block(else_body, vars, code)?;
                code.push(Instr::Label(end));
            }
            // Continue vuelve a la condición; break salta a la salida.
            Stmt::While { cond, body } => {
                let start = self.new_label("while_start");
                let end = self.new_label("while_end");
                code.push(Instr::Label(start.clone()));
                self.expr(cond, vars, code)?;
                code.push(Instr::JumpIfFalse(end.clone()));
                self.loops
                    .push((start.clone(), end.clone(), self.cleanups.len()));
                self.block(body, vars, code)?;
                self.loops.pop();
                code.push(Instr::Jump(start));
                code.push(Instr::Label(end));
            }
            // Continue pasa por el incremento y break sale sin incrementarlo.
            Stmt::ForRange {
                var,
                start,
                stop,
                body,
            } => {
                self.expr(start, vars, code)?;
                code.push(Instr::Store(var.clone()));
                let top = self.new_label("for_start");
                let cont = self.new_label("for_continue");
                let end = self.new_label("for_end");
                code.push(Instr::Label(top.clone()));
                code.push(Instr::Load(var.clone()));
                self.expr(stop, vars, code)?;
                code.push(Instr::CmpLt);
                code.push(Instr::JumpIfFalse(end.clone()));
                self.loops
                    .push((cont.clone(), end.clone(), self.cleanups.len()));
                self.block(body, vars, code)?;
                self.loops.pop();
                code.push(Instr::Label(cont));
                code.push(Instr::Load(var.clone()));
                code.push(Instr::PushI64(1));
                code.push(Instr::Add);
                code.push(Instr::Store(var.clone()));
                code.push(Instr::Jump(top));
                code.push(Instr::Label(end));
            }
            Stmt::Break => {
                let (_, b, depth) = self
                    .loops
                    .last()
                    .cloned()
                    .ok_or("romper fuera de un bucle")?;
                self.cleanup_to(depth, vars, code)?;
                code.push(Instr::Jump(b));
            }
            Stmt::Continue => {
                let (c, _, depth) = self
                    .loops
                    .last()
                    .cloned()
                    .ok_or("continuar fuera de un bucle")?;
                self.cleanup_to(depth, vars, code)?;
                code.push(Instr::Jump(c));
            }
            Stmt::Pass => {}
        }
        Ok(())
    }
    /// Emite instrucciones para evaluar `e` y apilar su resultado en `code`.
    /// Los operandos y argumentos se evalúan de izquierda a derecha. `vars` se
    /// transmite recursivamente. And/or usan saltos para cortocircuito y producen 0/1.
    /// Las cadenas apilan un descriptor UTF-8; los flotantes apilan sus bits IEEE-754.
    /// print solo se permite como sentencia; stmt elige la rutina según el tipo.
    fn expr(
        &mut self,
        e: &Expr,
        vars: &HashMap<String, TypeName>,
        code: &mut Vec<Instr>,
    ) -> Result<(), String> {
        match e {
            Expr::Member { object, name } => {
                let call =
                    crate::st::objects::member_call(object, "get", name, &[], vars, self.fns)?;
                self.expr(&call, vars, code)?;
            }
            Expr::MethodCall { object, name, args } => {
                let call =
                    crate::st::objects::member_call(object, "method", name, args, vars, self.fns)?;
                self.expr(&call, vars, code)?;
            }
            Expr::New(call) => self.expr(call, vars, code)?,
            Expr::Record { values, .. } => {
                code.push(Instr::Collection(CollectionOp::New));
                for value in values {
                    self.expr(value, vars, code)?;
                    code.push(Instr::PushI64(runtime_tag(infer_expr(
                        value, vars, self.fns,
                    )?)?));
                    code.push(Instr::Collection(CollectionOp::AppendTagged));
                }
            }
            Expr::RecordSlot { object, index, .. } => {
                self.expr(object, vars, code)?;
                code.push(Instr::PushI64(*index as i64));
                code.push(Instr::Collection(CollectionOp::ListGet));
            }

            Expr::TextOp { name, args } => {
                for arg in args {
                    self.expr(arg, vars, code)?;
                }
                if name == "tipo" {
                    let argument_type = infer_expr(&args[0], vars, self.fns)?;
                    if argument_type == TypeName::Dynamic {
                        // Una posición mixta conserva su etiqueta; el tipo se consulta
                        // en ejecución sin volver a evaluar ni reinterpretar el valor.
                        code.push(Instr::Scalar(ScalarOp::DynamicType));
                    } else {
                        code.push(Instr::Pop);
                        let id = self.out.strings.len();
                        self.out
                            .strings
                            .push(argument_type.st_name().as_bytes().to_vec());
                        code.push(Instr::PushText(id));
                    }
                } else if matches!(name.as_str(), "entero" | "flotante" | "booleano") {
                    let ty = infer_expr(&args[0], vars, self.fns)?;
                    match (name.as_str(), ty) {
                        ("entero", TypeName::Dynamic) => {
                            code.push(Instr::Scalar(ScalarOp::DynamicInt))
                        }
                        ("entero", TypeName::Float) => code.push(Instr::Scalar(ScalarOp::FloatInt)),
                        ("entero", TypeName::Str) => code.push(Instr::Scalar(ScalarOp::TextInt)),
                        ("flotante", TypeName::Dynamic) => {
                            code.push(Instr::Scalar(ScalarOp::DynamicFloat))
                        }
                        ("flotante", TypeName::Int | TypeName::Bool) => {
                            code.push(Instr::Scalar(ScalarOp::IntFloat))
                        }
                        ("flotante", TypeName::Str) => {
                            code.push(Instr::Scalar(ScalarOp::TextFloat))
                        }
                        ("booleano", TypeName::Dynamic) => {
                            code.push(Instr::Scalar(ScalarOp::DynamicBool))
                        }
                        ("booleano", TypeName::Float) => {
                            code.push(Instr::Scalar(ScalarOp::FloatBool))
                        }
                        ("booleano", TypeName::Str) => {
                            code.push(Instr::Scalar(ScalarOp::TextLength));
                            code.push(Instr::Not);
                            code.push(Instr::Not);
                        }
                        ("booleano", ty) if ty.is_collection() => {
                            code.push(Instr::Collection(CollectionOp::Length));
                            code.push(Instr::Not);
                            code.push(Instr::Not);
                        }
                        ("booleano", _) => {
                            code.push(Instr::Not);
                            code.push(Instr::Not);
                        }
                        _ => {} // Identidad: entero/booleano → entero o flotante → flotante.
                    }
                } else if let Some(native) = crate::st::stdlib::native(name) {
                    // Mismo registro de firmas que semantic; cada operación deja una palabra.
                    code.push(Instr::Scalar(native.op));
                } else if name == "cadena" {
                    let ty = infer_expr(&args[0], vars, self.fns)?;
                    if ty == TypeName::Dynamic {
                        code.push(Instr::Scalar(ScalarOp::DynamicText));
                    } else {
                        Self::text_conversion(ty, code);
                    }
                } else {
                    code.push(Instr::Scalar(if name == "entrada" {
                        if args.is_empty() {
                            ScalarOp::Input
                        } else {
                            ScalarOp::InputPrompt
                        }
                    } else if name == "abrir" {
                        ScalarOp::ReadFile
                    } else if name == "escribir" {
                        ScalarOp::WriteFile
                    } else if name == "buscar" {
                        ScalarOp::TextFind
                    } else {
                        ScalarOp::TextReplace
                    }));
                }
            }

            Expr::Null => code.push(Instr::PushI64(0)),
            Expr::Set(values) => {
                code.push(Instr::Collection(CollectionOp::New));
                for value in values {
                    self.expr(value, vars, code)?;
                    code.push(Instr::Collection(CollectionOp::SetAdd));
                }
            }
            Expr::List(values) | Expr::Tuple(values) => {
                code.push(Instr::Collection(CollectionOp::New));
                for value in values {
                    self.expr(value, vars, code)?;
                    let value_type = infer_expr(value, vars, self.fns)?;
                    if value_type == TypeName::Dynamic {
                        // Indexar otra lista mixta ya produce una caja con etiqueta
                        // y contenido; se copian ambas palabras, sin crear una caja
                        // dentro de otra ni perder el tipo real del elemento.
                        code.push(Instr::Collection(CollectionOp::AppendDynamic));
                    } else {
                        code.push(Instr::PushI64(runtime_tag(value_type)?));
                        code.push(Instr::Collection(CollectionOp::AppendTagged));
                    }
                }
            }
            Expr::Dict(values) => {
                code.push(Instr::Collection(CollectionOp::New));
                for (key, value) in values {
                    self.expr(key, vars, code)?;
                    self.expr(value, vars, code)?;
                    code.push(Instr::Collection(CollectionOp::TextDictSet));
                }
            }
            Expr::Index { collection, key } => {
                if infer_expr(collection, vars, self.fns)? == TypeName::Str {
                    self.expr(collection, vars, code)?;
                    self.expr(key, vars, code)?;
                    code.push(Instr::Scalar(ScalarOp::TextIndex));
                    return Ok(());
                }
                let dict = self.index(collection, key, vars, code)?;
                code.push(Instr::Collection(if dict {
                    CollectionOp::TextDictGet
                } else if infer_expr(collection, vars, self.fns)?.element()
                    == Some(TypeName::Dynamic)
                {
                    CollectionOp::ListGetDynamic
                } else {
                    CollectionOp::ListGet
                }));
            }
            Expr::Length(collection) => {
                self.expr(collection, vars, code)?;
                if infer_expr(collection, vars, self.fns)? == TypeName::Str {
                    code.push(Instr::Scalar(ScalarOp::TextLength));
                } else {
                    code.push(Instr::Collection(CollectionOp::Length));
                }
            }
            Expr::Append { collection, value } => {
                self.expr(collection, vars, code)?;
                self.expr(value, vars, code)?;
                let collection_type = infer_expr(collection, vars, self.fns)?;
                let value_type = infer_expr(value, vars, self.fns)?;
                let operation = if collection_type == TypeName::Set {
                    CollectionOp::SetAdd
                } else if value_type == TypeName::Dynamic {
                    CollectionOp::AppendDynamic
                } else {
                    code.push(Instr::PushI64(runtime_tag(value_type)?));
                    CollectionOp::AppendTagged
                };
                code.push(Instr::Collection(operation));
                code.push(Instr::Pop);
                code.push(Instr::PushI64(0));
            }
            Expr::Float(n) => code.push(Instr::PushI64(n.to_bits() as i64)),
            Expr::Int(n) => code.push(Instr::PushI64(*n)),
            Expr::Bool(b) => code.push(Instr::PushI64(if *b { 1 } else { 0 })),
            Expr::Str(text) => {
                let id = self.out.strings.len();
                self.out.strings.push(text.as_bytes().to_vec());
                code.push(Instr::PushText(id));
            }
            Expr::Var(n) => code.push(Instr::Load(n.clone())),
            Expr::Unary { op, expr } => {
                self.expr(expr, vars, code)?;
                match op {
                    UnaryOp::Neg => {
                        code.push(if infer_expr(expr, vars, self.fns)? == TypeName::Float {
                            Instr::Scalar(ScalarOp::FloatNeg)
                        } else {
                            Instr::Neg
                        })
                    }
                    UnaryOp::Pos => {}
                    UnaryOp::Not => code.push(Instr::Not),
                }
            }
            // El primer operando falso evita evaluar lo restante y produce cero.
            Expr::Binary {
                op: BinOp::And,
                left,
                right,
            } => {
                let f = self.new_label("and_false");
                let end = self.new_label("and_end");
                self.expr(left, vars, code)?;
                code.push(Instr::JumpIfFalse(f.clone()));
                self.expr(right, vars, code)?;
                code.push(Instr::JumpIfFalse(f.clone()));
                code.push(Instr::PushI64(1));
                code.push(Instr::Jump(end.clone()));
                code.push(Instr::Label(f));
                code.push(Instr::PushI64(0));
                code.push(Instr::Label(end));
            }
            // El primer operando verdadero evita evaluar lo restante y produce uno.
            Expr::Binary {
                op: BinOp::Or,
                left,
                right,
            } => {
                let t = self.new_label("or_true");
                let end = self.new_label("or_end");
                self.expr(left, vars, code)?;
                code.push(Instr::JumpIfTrue(t.clone()));
                self.expr(right, vars, code)?;
                code.push(Instr::JumpIfTrue(t.clone()));
                code.push(Instr::PushI64(0));
                code.push(Instr::Jump(end.clone()));
                code.push(Instr::Label(t));
                code.push(Instr::PushI64(1));
                code.push(Instr::Label(end));
            }
            Expr::Binary { op, left, right } => {
                if matches!(op, BinOp::Add) && infer_expr(e, vars, self.fns)? == TypeName::Str {
                    self.expr(left, vars, code)?;
                    Self::text_conversion(infer_expr(left, vars, self.fns)?, code);
                    self.expr(right, vars, code)?;
                    Self::text_conversion(infer_expr(right, vars, self.fns)?, code);
                    code.push(Instr::Scalar(ScalarOp::TextConcat));
                    return Ok(());
                }
                self.expr(left, vars, code)?;
                self.expr(right, vars, code)?;
                let ty = infer_expr(left, vars, self.fns)?;
                if ty == TypeName::Float {
                    code.push(Instr::Scalar(Self::float_op(*op)?));
                    return Ok(());
                }
                if ty == TypeName::Str {
                    code.push(Instr::Scalar(if matches!(op, BinOp::Add) {
                        ScalarOp::TextConcat
                    } else {
                        ScalarOp::TextEq
                    }));
                    if matches!(op, BinOp::Ne) {
                        code.push(Instr::Not);
                    }
                    return Ok(());
                }
                code.push(match op {
                    BinOp::Add => Instr::Add,
                    BinOp::Sub => Instr::Sub,
                    BinOp::Mul => Instr::Mul,
                    BinOp::Div => Instr::Div,
                    BinOp::Mod => Instr::Mod,
                    BinOp::Eq | BinOp::Is => Instr::CmpEq,
                    BinOp::Ne => Instr::CmpNe,
                    BinOp::Lt => Instr::CmpLt,
                    BinOp::Le => Instr::CmpLe,
                    BinOp::Gt => Instr::CmpGt,
                    BinOp::Ge => Instr::CmpGe,
                    BinOp::And | BinOp::Or => unreachable!(),
                });
            }
            Expr::Call { name, args } => {
                if name == "print" {
                    return Err("la impresión solo se permite como sentencia".into());
                }
                if name == "len" && !self.fns.contains_key(name) {
                    if args.len() != 1 {
                        return Err("len requiere exactamente un argumento".into());
                    }
                    let ty = infer_expr(&args[0], vars, self.fns)?;
                    self.expr(&args[0], vars, code)?;
                    if ty == TypeName::Str {
                        code.push(Instr::Scalar(ScalarOp::TextLength));
                    } else {
                        code.push(Instr::Collection(CollectionOp::Length));
                    }
                    return Ok(());
                }
                if name == "sqrt" && !self.fns.contains_key(name) {
                    if args.len() != 1 {
                        return Err("sqrt requiere exactamente un argumento".into());
                    }
                    self.expr(&args[0], vars, code)?;
                    if infer_expr(&args[0], vars, self.fns)? == TypeName::Int {
                        code.push(Instr::Scalar(ScalarOp::IntFloat));
                    }
                    code.push(Instr::Scalar(ScalarOp::SquareRoot));
                    return Ok(());
                }
                if name == "abs" && !self.fns.contains_key(name) {
                    if args.len() != 1 {
                        return Err("abs requiere exactamente un argumento".into());
                    }
                    let ty = infer_expr(&args[0], vars, self.fns)?;
                    let tmp = self.temporary();
                    self.expr(&args[0], vars, code)?;
                    code.push(Instr::Store(tmp.clone()));
                    code.push(Instr::Load(tmp.clone()));
                    code.push(Instr::Load(tmp.clone()));
                    code.push(Instr::PushI64(0));
                    let end = self.new_label("abs_end");
                    if ty == TypeName::Float {
                        code.push(Instr::Scalar(ScalarOp::FloatLt));
                        code.push(Instr::JumpIfFalse(end.clone()));
                        code.push(Instr::Scalar(ScalarOp::FloatNeg));
                    } else {
                        code.push(Instr::CmpLt);
                        code.push(Instr::JumpIfFalse(end.clone()));
                        code.push(Instr::Neg);
                    }
                    code.push(Instr::Label(end));
                    return Ok(());
                }
                if name == "__contains__" && !self.fns.contains_key(name) {
                    if args.len() != 2 {
                        return Err("__contains__ requiere exactamente dos argumentos".into());
                    }
                    let collection = &args[1];
                    let ty = infer_expr(collection, vars, self.fns)?;
                    self.expr(collection, vars, code)?;
                    self.expr(&args[0], vars, code)?;
                    let value_type = infer_expr(&args[0], vars, self.fns)?;
                    let operation = if ty.element() == Some(TypeName::Dynamic) && ty.is_list() {
                        if value_type == TypeName::Dynamic {
                            CollectionOp::ContainsListDynamic
                        } else {
                            code.push(Instr::PushI64(runtime_tag(value_type)?));
                            CollectionOp::ContainsListTagged
                        }
                    } else if ty.is_list() || ty == TypeName::Tuple || ty == TypeName::Set {
                        CollectionOp::ContainsList
                    } else {
                        CollectionOp::ContainsDict
                    };
                    code.push(Instr::Collection(operation));
                    return Ok(());
                }
                for a in args {
                    self.expr(a, vars, code)?;
                }
                code.push(Instr::Call {
                    name: name.clone(),
                    argc: args.len(),
                });
            }
        }
        Ok(())
    }
}

/// Etiqueta estable almacenada junto al valor de cada lista. Las listas tipadas
/// también la escriben, por lo que pueden prestarse como `lista[mixto]` sin copiar.
fn runtime_tag(ty: TypeName) -> Result<i64, String> {
    Ok(match ty {
        TypeName::Int => 1,
        TypeName::Float => 2,
        TypeName::Bool => 3,
        TypeName::Str => 4,
        TypeName::Null => 5,
        ty if ty.is_list() => 6,
        ty if ty.is_dict() => 7,
        TypeName::Tuple => 8,
        TypeName::Set => 9,
        TypeName::Object(_) => 10,
        TypeName::Dynamic => {
            return Err("un valor mixto debe copiar su etiqueta de ejecución".into())
        }
        _ => return Err(format!("{} no puede guardarse en una lista", ty.as_str())),
    })
}
