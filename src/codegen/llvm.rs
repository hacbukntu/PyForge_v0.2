//! Backend LLVM 21 de PyForge construido sobre Inkwell.
//!
//! Convierte la IR de pila verificada en funciones LLVM que usan palabras de 64
//! bits, la misma ABI del backend histórico. Los valores temporales viven primero
//! en `alloca`; el pipeline de LLVM los promueve a SSA mediante sus optimizaciones.
//! El runtime ensamblador se concatena al resultado para conservar texto, listas,
//! archivos, teclado y recursos sin introducir dos implementaciones incompatibles.
//!
//! Las excepciones actuales restauran `rsp`/`rbp` y saltan a una etiqueta nativa.
//! Mientras se migra esa ABI a unwinding propio de LLVM, esos programas se emiten
//! como ensamblador de módulo en línea. Así el backend único conserva cobertura y
//! pasa por LLVM sin cambiar el comportamiento probado del lenguaje.

use crate::{
    codegen::llvm_compat::{emit_linux_asm, emit_linux_runtime_asm},
    ir::{FunctionIr, Instr, ProgramIr},
};
use inkwell::{
    attributes::AttributeLoc,
    basic_block::BasicBlock,
    builder::Builder,
    context::Context,
    module::{Linkage, Module},
    passes::PassBuilderOptions,
    targets::{CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine},
    types::BasicMetadataTypeEnum,
    values::{BasicMetadataValueEnum, FunctionValue, IntValue, PointerValue},
    IntPredicate, OptimizationLevel,
};
use std::collections::HashMap;

/// Textos observables producidos por LLVM para diagnóstico y enlace.
pub struct LlvmArtifacts {
    pub assembly: String,
    pub llvm_ir: String,
    pub compatibility_mode: bool,
}

/// Genera ensamblador GNU para el host mediante LLVM y adjunta el runtime ST.
/// El runtime actual limita el resultado final a Linux x86-64, aunque la capa LLVM
/// ya restringe sus targets compilados a x86 y AArch64 para la siguiente etapa.
pub fn emit_linux_asm_llvm(program: &ProgramIr, optimize: bool) -> Result<String, String> {
    Ok(emit_linux_artifacts(program, optimize)?.assembly)
}

/// Produce tanto LLVM IR como ensamblador para que `emit-llvm` pueda inspeccionar
/// lo que realmente recibe el optimizador sin reconstruir el módulo una segunda vez.
pub fn emit_linux_artifacts(program: &ProgramIr, optimize: bool) -> Result<LlvmArtifacts, String> {
    Target::initialize_x86(&InitializationConfig::default());
    let triple = TargetMachine::get_default_triple();
    let triple_text = triple.as_str().to_string_lossy();
    if !triple_text.starts_with("x86_64") || !triple_text.contains("linux") {
        return Err(format!(
            "el runtime LLVM de ST admite por ahora x86_64 Linux; objetivo detectado: {triple_text}"
        ));
    }
    let target = Target::from_triple(&triple)
        .map_err(|error| format!("LLVM no reconoce el objetivo {triple_text}: {error}"))?;
    let level = if optimize {
        OptimizationLevel::Aggressive
    } else {
        OptimizationLevel::None
    };
    let machine = target
        .create_target_machine(
            &triple,
            "generic",
            "",
            level,
            RelocMode::Static,
            CodeModel::Small,
        )
        .ok_or_else(|| format!("LLVM no pudo crear la máquina objetivo {triple_text}"))?;

    if uses_native_exception_abi(program) {
        return emit_compatibility_module(program, &machine, &triple);
    }

    let context = Context::create();
    let module = context.create_module("pyforge_st");
    module.set_triple(&triple);
    module.set_data_layout(&machine.get_target_data().get_data_layout());
    lower_module(&context, &module, program)?;
    module
        .verify()
        .map_err(|error| format!("verificación LLVM: {error}"))?;
    if optimize {
        module
            .run_passes("default<O2>", &machine, PassBuilderOptions::create())
            .map_err(|error| format!("optimización LLVM: {error}"))?;
        module
            .verify()
            .map_err(|error| format!("verificación LLVM posterior a optimizar: {error}"))?;
    }
    let llvm_ir = module.print_to_string().to_string();
    let buffer = machine
        .write_to_memory_buffer(&module, FileType::Assembly)
        .map_err(|error| format!("emisión LLVM: {error}"))?;
    let mut assembly = String::from_utf8(buffer.as_slice().to_vec())
        .map_err(|_| "LLVM produjo ensamblador que no es UTF-8".to_string())?;
    while assembly.ends_with('\0') {
        assembly.pop();
    }
    assembly.push('\n');
    assembly.push_str(&emit_linux_runtime_asm(program)?);
    Ok(LlvmArtifacts {
        assembly,
        llvm_ir,
        compatibility_mode: false,
    })
}

/// Detecta las instrucciones cuya ABI todavía necesita restaurar el marco exacto
/// generado por el backend ensamblador. El resto del programa usa lowering LLVM real.
fn uses_native_exception_abi(program: &ProgramIr) -> bool {
    program
        .functions
        .iter()
        .chain(program.main.iter())
        .flat_map(|function| &function.code)
        .any(|instruction| {
            matches!(
                instruction,
                Instr::TryBegin { .. }
                    | Instr::TryEnd { .. }
                    | Instr::ExceptionValue
                    | Instr::Throw
            )
        })
}

/// Encapsula el backend probado como ensamblador de módulo LLVM. LLVM valida el
/// objetivo y produce el texto final; esto mantiene excepciones operativas hasta
/// que su unwinder deje de depender de las posiciones concretas de `rsp` y `rbp`.
fn emit_compatibility_module(
    program: &ProgramIr,
    machine: &TargetMachine,
    triple: &inkwell::targets::TargetTriple,
) -> Result<LlvmArtifacts, String> {
    let context = Context::create();
    let module = context.create_module("pyforge_st_compat");
    module.set_triple(triple);
    module.set_data_layout(&machine.get_target_data().get_data_layout());
    // El ensamblador integrado de LLVM usa la forma inmediata sin `OFFSET FLAT`,
    // mientras GNU as necesita esa marca en el backend histórico. La sustitución
    // solo afecta la copia que recibe LLVM y conserva intacta la salida nativa.
    let native = emit_linux_asm(program)?.replace("OFFSET FLAT:", "OFFSET ");
    module.set_inline_assembly(&native);
    let llvm_ir = module.print_to_string().to_string();
    let buffer = machine
        .write_to_memory_buffer(&module, FileType::Assembly)
        .map_err(|error| format!("emisión LLVM de compatibilidad: {error}"))?;
    let mut assembly = String::from_utf8(buffer.as_slice().to_vec())
        .map_err(|_| "LLVM produjo ensamblador que no es UTF-8".to_string())?;
    while assembly.ends_with('\0') {
        assembly.pop();
    }
    Ok(LlvmArtifacts {
        assembly,
        llvm_ir,
        compatibility_mode: true,
    })
}

/// Declara primero todas las funciones para admitir llamadas hacia delante y
/// recursión; después genera sus bloques e instrucciones.
fn lower_module<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    program: &ProgramIr,
) -> Result<(), String> {
    let i64_type = context.i64_type();
    let mut functions = HashMap::new();
    for function in program.functions.iter().chain(program.main.iter()) {
        let parameters: Vec<BasicMetadataTypeEnum> = (0..function.params.len())
            .map(|_| i64_type.into())
            .collect();
        let value = module.add_function(
            &symbol(&function.name),
            i64_type.fn_type(&parameters, false),
            Some(Linkage::External),
        );
        // ST limita la recursión mediante la pila real. Desactivar tail calls evita
        // que O2 transforme una recursión infinita en un bucle que eluda el límite.
        value.add_attribute(
            AttributeLoc::Function,
            context.create_string_attribute("disable-tail-calls", "true"),
        );
        functions.insert(function.name.clone(), value);
    }
    for function in program.functions.iter().chain(program.main.iter()) {
        lower_function(context, module, function, &functions, &program.strings)?;
    }
    Ok(())
}

/// Traduce una función de pila a bloques LLVM. Cada posición de la pila verificada
/// tiene un `alloca`; por eso los saltos pueden converger sin fabricar PHI a mano.
fn lower_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    source: &FunctionIr,
    functions: &HashMap<String, FunctionValue<'ctx>>,
    strings: &[Vec<u8>],
) -> Result<(), String> {
    let function = functions[&source.name];
    let builder = context.create_builder();
    let i64_type = context.i64_type();
    let (depths, maximum) = crate::verify::verify(source)?;
    let entry = context.append_basic_block(function, "entrada");
    builder.position_at_end(entry);

    let mut variables = source.params.clone();
    for local in &source.locals {
        if !variables.contains(local) {
            variables.push(local.clone());
        }
    }
    let mut local_slots = HashMap::new();
    for name in variables {
        let slot = builder
            .build_alloca(i64_type, &format!("local.{name}"))
            .map_err(builder_error)?;
        builder
            .build_store(slot, i64_type.const_zero())
            .map_err(builder_error)?;
        local_slots.insert(name, slot);
    }
    for (index, name) in source.params.iter().enumerate() {
        let argument = function
            .get_nth_param(index as u32)
            .ok_or_else(|| format!("LLVM: falta el parámetro {index} de {}", source.name))?;
        builder
            .build_store(local_slots[name], argument)
            .map_err(builder_error)?;
    }
    let mut stack_slots = Vec::with_capacity(maximum.max(1));
    for index in 0..maximum.max(1) {
        stack_slots.push(
            builder
                .build_alloca(i64_type, &format!("pila.{index}"))
                .map_err(builder_error)?,
        );
    }

    // El runtime fija `pf_stack_floor` antes de entrar en `pf_main`. La llamada
    // comprueba también funciones LLVM recursivas, no solo la ruta de excepciones.
    let _ = call_word_function(context, module, &builder, "pf_llvm_stack_check", Vec::new())?;

    let mut blocks: Vec<Option<BasicBlock<'ctx>>> = vec![None; source.code.len()];
    for (index, depth) in depths.iter().enumerate() {
        if depth.is_some() {
            blocks[index] = Some(context.append_basic_block(function, &format!("ir.{index}")));
        }
    }
    let first = blocks
        .iter()
        .flatten()
        .next()
        .copied()
        .ok_or_else(|| format!("LLVM: la función {} no tiene instrucciones", source.name))?;
    builder
        .build_unconditional_branch(first)
        .map_err(builder_error)?;

    let labels: HashMap<&str, usize> = source
        .code
        .iter()
        .enumerate()
        .filter_map(|(index, instruction)| match instruction {
            Instr::Label(name) => Some((name.as_str(), index)),
            _ => None,
        })
        .collect();

    for (index, instruction) in source.code.iter().enumerate() {
        let Some(depth) = depths[index] else { continue };
        let block = blocks[index].expect("bloque de instrucción alcanzable");
        builder.position_at_end(block);
        let next = || {
            blocks
                .get(index + 1)
                .and_then(|block| *block)
                .ok_or_else(|| format!("LLVM: falta continuación después de {index}"))
        };
        let jump_target = |name: &str| {
            let destination = *labels
                .get(name)
                .ok_or_else(|| format!("LLVM: etiqueta desconocida {name}"))?;
            blocks[destination].ok_or_else(|| format!("LLVM: etiqueta inalcanzable {name}"))
        };
        let load_stack =
            |slot: usize, name: &str| load_i64(&builder, i64_type, stack_slots[slot], name);
        let store_stack = |slot: usize, value: IntValue<'ctx>| {
            builder
                .build_store(stack_slots[slot], value)
                .map(|_| ())
                .map_err(builder_error)
        };

        match instruction {
            Instr::PushI64(value) => {
                store_stack(depth, i64_type.const_int(*value as u64, true))?;
                branch(&builder, next()?)?;
            }
            Instr::PushText(id) => {
                let global = external_global(module, i64_type, &format!("pf_text_{id}"));
                let address = builder
                    .build_ptr_to_int(global.as_pointer_value(), i64_type, "texto")
                    .map_err(builder_error)?;
                store_stack(depth, address)?;
                branch(&builder, next()?)?;
            }
            Instr::Load(name) => {
                let slot = *local_slots
                    .get(name)
                    .ok_or_else(|| format!("LLVM: variable local desconocida {name}"))?;
                store_stack(depth, load_i64(&builder, i64_type, slot, "cargar")?)?;
                branch(&builder, next()?)?;
            }
            Instr::Store(name) => {
                let slot = *local_slots
                    .get(name)
                    .ok_or_else(|| format!("LLVM: variable local desconocida {name}"))?;
                builder
                    .build_store(slot, load_stack(depth - 1, "guardar")?)
                    .map_err(builder_error)?;
                branch(&builder, next()?)?;
            }
            Instr::Collection(operation) => {
                let (name, count) = operation.runtime();
                let result = call_word_function(
                    context,
                    module,
                    &builder,
                    name,
                    arguments(&builder, i64_type, &stack_slots, depth, count)?,
                )?;
                store_stack(depth - count, result)?;
                branch(&builder, next()?)?;
            }
            Instr::Scalar(operation) => {
                let (name, count) = operation.runtime();
                let result = call_word_function(
                    context,
                    module,
                    &builder,
                    name,
                    arguments(&builder, i64_type, &stack_slots, depth, count)?,
                )?;
                store_stack(depth - count, result)?;
                branch(&builder, next()?)?;
            }
            Instr::Add | Instr::Sub | Instr::Mul | Instr::Div | Instr::Mod => {
                let name = match instruction {
                    Instr::Add => "pf_llvm_add",
                    Instr::Sub => "pf_llvm_sub",
                    Instr::Mul => "pf_llvm_mul",
                    Instr::Div => "pf_llvm_div",
                    Instr::Mod => "pf_llvm_mod",
                    _ => unreachable!(),
                };
                let result = call_word_function(
                    context,
                    module,
                    &builder,
                    name,
                    vec![
                        load_stack(depth - 2, "izquierda")?,
                        load_stack(depth - 1, "derecha")?,
                    ],
                )?;
                store_stack(depth - 2, result)?;
                branch(&builder, next()?)?;
            }
            Instr::Neg => {
                let result = call_word_function(
                    context,
                    module,
                    &builder,
                    "pf_llvm_neg",
                    vec![load_stack(depth - 1, "negando")?],
                )?;
                store_stack(depth - 1, result)?;
                branch(&builder, next()?)?;
            }
            Instr::Not => {
                let value = load_stack(depth - 1, "no")?;
                let result = builder
                    .build_int_compare(IntPredicate::EQ, value, i64_type.const_zero(), "no")
                    .and_then(|value| builder.build_int_z_extend(value, i64_type, "no.i64"))
                    .map_err(builder_error)?;
                store_stack(depth - 1, result)?;
                branch(&builder, next()?)?;
            }
            Instr::CmpEq
            | Instr::CmpNe
            | Instr::CmpLt
            | Instr::CmpLe
            | Instr::CmpGt
            | Instr::CmpGe => {
                let predicate = match instruction {
                    Instr::CmpEq => IntPredicate::EQ,
                    Instr::CmpNe => IntPredicate::NE,
                    Instr::CmpLt => IntPredicate::SLT,
                    Instr::CmpLe => IntPredicate::SLE,
                    Instr::CmpGt => IntPredicate::SGT,
                    Instr::CmpGe => IntPredicate::SGE,
                    _ => unreachable!(),
                };
                let result = builder
                    .build_int_compare(
                        predicate,
                        load_stack(depth - 2, "comparar.izquierda")?,
                        load_stack(depth - 1, "comparar.derecha")?,
                        "comparacion",
                    )
                    .and_then(|value| builder.build_int_z_extend(value, i64_type, "bool.i64"))
                    .map_err(builder_error)?;
                store_stack(depth - 2, result)?;
                branch(&builder, next()?)?;
            }
            Instr::Label(_) => branch(&builder, next()?)?,
            Instr::Jump(name) => branch(&builder, jump_target(name)?)?,
            Instr::JumpIfFalse(name) | Instr::JumpIfTrue(name) => {
                let condition = builder
                    .build_int_compare(
                        IntPredicate::NE,
                        load_stack(depth - 1, "condicion")?,
                        i64_type.const_zero(),
                        "condicion.bool",
                    )
                    .map_err(builder_error)?;
                let target = jump_target(name)?;
                let fallthrough = next()?;
                let (yes, no) = if matches!(instruction, Instr::JumpIfTrue(_)) {
                    (target, fallthrough)
                } else {
                    (fallthrough, target)
                };
                builder
                    .build_conditional_branch(condition, yes, no)
                    .map_err(builder_error)?;
            }
            Instr::Call { name, argc } => {
                let called = *functions
                    .get(name)
                    .ok_or_else(|| format!("LLVM: función desconocida {name}"))?;
                let values = arguments(&builder, i64_type, &stack_slots, depth, *argc)?;
                let metadata: Vec<BasicMetadataValueEnum> =
                    values.into_iter().map(Into::into).collect();
                let result = builder
                    .build_call(called, &metadata, "llamada")
                    .map_err(builder_error)?
                    .try_as_basic_value()
                    .basic()
                    .ok_or_else(|| format!("LLVM: {name} no devolvió una palabra"))?
                    .into_int_value();
                store_stack(depth - argc, result)?;
                branch(&builder, next()?)?;
            }
            Instr::Pop => branch(&builder, next()?)?,
            Instr::PrintInt | Instr::PrintBool => {
                let name = if matches!(instruction, Instr::PrintInt) {
                    "pf_print_i64"
                } else {
                    "pf_print_bool"
                };
                let _ = call_word_function(
                    context,
                    module,
                    &builder,
                    name,
                    vec![load_stack(depth - 1, "imprimir")?],
                )?;
                branch(&builder, next()?)?;
            }
            Instr::PrintStr(id) => {
                let bytes = strings
                    .get(*id)
                    .ok_or_else(|| format!("LLVM: falta el literal de texto {id}"))?;
                let global = external_global(module, context.i8_type(), &format!("pf_str_{id}"));
                let address = builder
                    .build_ptr_to_int(global.as_pointer_value(), i64_type, "literal")
                    .map_err(builder_error)?;
                let _ = call_word_function(
                    context,
                    module,
                    &builder,
                    "pf_print_literal",
                    vec![address, i64_type.const_int(bytes.len() as u64, false)],
                )?;
                branch(&builder, next()?)?;
            }
            Instr::ReturnValue => {
                let value = load_stack(depth - 1, "retorno")?;
                builder.build_return(Some(&value)).map_err(builder_error)?;
            }
            Instr::ReturnVoid => {
                builder
                    .build_return(Some(&i64_type.const_zero()))
                    .map_err(builder_error)?;
            }
            Instr::TryBegin { .. }
            | Instr::TryEnd { .. }
            | Instr::ExceptionValue
            | Instr::Throw => {
                return Err("LLVM: la excepción debió usar el módulo de compatibilidad".into())
            }
        }
    }
    Ok(())
}

/// Carga una palabra desde un `alloca` opaco de LLVM 21.
fn load_i64<'ctx>(
    builder: &Builder<'ctx>,
    ty: inkwell::types::IntType<'ctx>,
    slot: PointerValue<'ctx>,
    name: &str,
) -> Result<IntValue<'ctx>, String> {
    builder
        .build_load(ty, slot, name)
        .map_err(builder_error)
        .map(|value| value.into_int_value())
}

/// Lee los argumentos desde las posiciones inferiores de la pila en orden ABI.
fn arguments<'ctx>(
    builder: &Builder<'ctx>,
    ty: inkwell::types::IntType<'ctx>,
    stack: &[PointerValue<'ctx>],
    depth: usize,
    count: usize,
) -> Result<Vec<IntValue<'ctx>>, String> {
    (depth - count..depth)
        .map(|index| load_i64(builder, ty, stack[index], "argumento"))
        .collect()
}

/// Declara o llama una función de runtime `i64(i64, ...)`.
fn call_word_function<'ctx>(
    context: &'ctx Context,
    module: &Module<'ctx>,
    builder: &Builder<'ctx>,
    name: &str,
    arguments: Vec<IntValue<'ctx>>,
) -> Result<IntValue<'ctx>, String> {
    let function = module.get_function(name).unwrap_or_else(|| {
        let types: Vec<BasicMetadataTypeEnum> = (0..arguments.len())
            .map(|_| context.i64_type().into())
            .collect();
        module.add_function(
            name,
            context.i64_type().fn_type(&types, false),
            Some(Linkage::External),
        )
    });
    let metadata: Vec<BasicMetadataValueEnum> = arguments.into_iter().map(Into::into).collect();
    builder
        .build_call(function, &metadata, name)
        .map_err(builder_error)?
        .try_as_basic_value()
        .basic()
        .ok_or_else(|| format!("LLVM: el runtime {name} no devolvió una palabra"))
        .map(|value| value.into_int_value())
}

/// Declara un símbolo de datos definido después por el runtime ensamblador.
fn external_global<'ctx, T: inkwell::types::BasicType<'ctx>>(
    module: &Module<'ctx>,
    ty: T,
    name: &str,
) -> inkwell::values::GlobalValue<'ctx> {
    module.get_global(name).unwrap_or_else(|| {
        let value = module.add_global(ty, None, name);
        value.set_linkage(Linkage::External);
        value
    })
}

/// Añade una rama y normaliza el error de Inkwell al diagnóstico de PyForge.
fn branch(builder: &Builder<'_>, target: BasicBlock<'_>) -> Result<(), String> {
    builder
        .build_unconditional_branch(target)
        .map(|_| ())
        .map_err(builder_error)
}

/// Mantiene los símbolos estables de la ABI del runtime Linux existente.
fn symbol(name: &str) -> String {
    if name == "__main__" {
        "pf_main".into()
    } else {
        format!("pf_user_{name}")
    }
}

/// Convierte cualquier error del builder en texto contextualizable por el driver.
fn builder_error(error: inkwell::builder::BuilderError) -> String {
    format!("constructor LLVM: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::TypeName;
    use crate::ir::FunctionIr;

    #[test]
    fn modulo_simple_contiene_main_y_runtime() {
        let program = ProgramIr {
            strings: vec![],
            functions: vec![],
            main: Some(FunctionIr {
                name: "__main__".into(),
                params: vec![],
                locals: vec![],
                return_type: TypeName::None,
                code: vec![Instr::ReturnVoid],
            }),
        };
        let assembly = emit_linux_asm_llvm(&program, false).expect("generar LLVM");
        assert!(assembly.contains("pf_main"));
        assert!(assembly.contains("pf_print_i64"));
    }
}
