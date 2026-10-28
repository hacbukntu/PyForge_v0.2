//! Runtime ensamblador Linux y compatibilidad temporal de excepciones para LLVM.
//!
//! Este módulo ya no es un backend seleccionable. `llvm.rs` usa sus datos y rutinas
//! de ejecución, y solicita `emit_linux_asm` únicamente cuando una función contiene
//! la ABI histórica de excepciones basada en `rsp`/`rbp`. Mantenerlo privado evita
//! presentar dos compiladores mientras esa última transición sigue verificada.
//! Traduce la pila intermedia a push/pop, asigna locales respecto a rbp y emite llamadas.
//! Incluye rutinas de impresión sin libc. Devuelve texto a driver; GNU as y ld
//! se encargan después del objeto y del ejecutable ELF.
use crate::ir::*;
use std::collections::HashMap;

/// Construye el ensamblador de `p`: sección de código, RUNTIME, _start, funciones
/// y datos de solo lectura. _start llama a pf_main y entrega su retorno a exit.
/// Las cadenas se emiten como bytes. Devuelve texto o un error de emit_fn.
pub fn emit_linux_asm(p: &ProgramIr) -> Result<String, String> {
    emit_linux_asm_parts(p, true)
}

/// Emite solamente el arranque, runtime y datos que necesita un módulo LLVM.
/// El backend LLVM genera `pf_main` y las funciones de usuario; después concatena
/// esta parte para conservar la ABI probada de colecciones, texto, IO y recursos.
pub fn emit_linux_runtime_asm(p: &ProgramIr) -> Result<String, String> {
    emit_linux_asm_parts(p, false)
}

/// Implementación compartida. `include_program` conserva el backend histórico o
/// deja que LLVM proporcione las funciones sin duplicar símbolos en el objeto.
fn emit_linux_asm_parts(p: &ProgramIr, include_program: bool) -> Result<String, String> {
    let mut s = String::new();
    s.push_str(".intel_syntax noprefix\n.section .text\n.global _start\n\n");
    // Exporta únicamente el puente de reserva que utiliza la ABI Rust.
    s.push_str(
        ".global pf_native_alloc\npf_native_alloc:\n    mov rsi, rdi\n    jmp pf_col_alloc\n",
    );
    // LLVM llama esta rutina al entrar en cada función. Comparar el puntero real
    // mantiene el presupuesto de recursión incluso después de optimizar a O2.
    s.push_str(
        ".global pf_llvm_stack_check\npf_llvm_stack_check:\n    mov rax, rsp\n    cmp rax, [rip+pf_stack_floor]\n    jb pf_stack_error\n    xor eax, eax\n    ret\n",
    );
    // No emite referencias Rust en programas que no usan la API tipada.
    if p.functions.iter().chain(p.main.iter()).any(|f| {
        f.code
            .iter()
            .any(|i| matches!(i, Instr::Scalar(op) if op.runtime().0.starts_with("pf_resource_")))
    }) {
        s.push_str(include_str!("resources.s"));
    }
    // Incorpora las rutinas de impresión al ejecutable, sin depender de libc.
    s.push_str(RUNTIME);
    s.push_str(include_str!("collections.s"));
    s.push_str(include_str!("scalars.s"));
    s.push_str(include_str!("dynamic.s"));
    s.push_str(include_str!("io.s"));
    s.push_str(include_str!("keyboard.s"));
    s.push_str(include_str!("stdlib.s"));
    s.push_str(include_str!("integraciones.s"));
    // Selección explícita al compilar: permite un entorno virtual sin instalar globalmente.
    // La ruta se emite como bytes para que nunca pueda inyectar ensamblador.
    let python = std::env::var("PYFORGE_PYTHON").unwrap_or_else(|_| "/usr/bin/python3".into());
    if !std::path::Path::new(&python).is_absolute() || python.len() > 4095 || python.contains('\0')
    {
        return Err(
            "PYFORGE_PYTHON debe ser una ruta absoluta sin NUL, de hasta 4095 bytes".into(),
        );
    }
    s.push_str("\n.section .rodata\npf_bridge_python: .byte ");
    s.push_str(
        &python
            .bytes()
            .map(|b| b.to_string())
            .collect::<Vec<_>>()
            .join(","),
    );
    s.push_str(",0\n");

    // El auxiliar se incrusta como datos; solo se ejecuta al usar integraciones.
    s.push_str("\n.section .rodata\npf_bridge_source:\n");
    let gui_hex = include_bytes!("../../runtime/gui.py")
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let helper = include_str!("../../runtime/integraciones.py").replace(
        "GUI_SOURCE = None",
        &format!("GUI_SOURCE = bytes.fromhex(\"{gui_hex}\").decode(\"utf-8\")"),
    );
    for chunk in helper.as_bytes().chunks(64) {
        s.push_str(".byte ");
        s.push_str(
            &chunk
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(","),
        );
        s.push('\n');
    }
    s.push_str(".byte 0\n.section .text\n");
    s.push_str(include_str!("conversions.s"));
    s.push_str(include_str!("exceptions.s"));
    s.push_str("\n_start:\n    mov rax, [rsp]\n    lea rax, [rsp+rax*8+16]\n    mov [rip+pf_bridge_env], rax\n    lea rax, [rsp-524288]\n    mov [rip+pf_stack_floor], rax\n    call pf_main\n    push rax\n    call pf_bridge_shutdown\n    pop rdi\n    mov rax, 60\n    syscall\n\n");
    if include_program {
        for f in &p.functions {
            emit_fn(&mut s, f, false, &p.strings)?;
        }
        if let Some(m) = &p.main {
            emit_fn(&mut s, m, true, &p.strings)?;
        }
    }
    s.push_str("\n.section .rodata\n");
    s.push_str("pf_true: .ascii \"True\\n\"\npf_true_len = . - pf_true\npf_false: .ascii \"False\\n\"\npf_false_len = . - pf_false\n");
    for (i, b) in p.strings.iter().enumerate() {
        s.push_str(&format!(
            ".balign 8\npf_text_{i}: .quad {}\npf_str_{i}:\n",
            b.len()
        ));
        if !b.is_empty() {
            s.push_str("    .byte ");
        }
        for (j, x) in b.iter().enumerate() {
            if j > 0 {
                s.push_str(", ");
            }
            s.push_str(&x.to_string());
        }
        s.push_str(&format!("\npf_str_{i}_len = . - pf_str_{i}\n"));
    }
    Ok(s)
}

/// Convierte `name` a símbolo: __main__ pasa a pf_main y el resto a pf_user_*.
/// Depende de nombres ya validados; no impide una colisión con __main__ de usuario.
fn sym(name: &str) -> String {
    if name == "__main__" {
        "pf_main".into()
    } else {
        format!("pf_user_{name}")
    }
}
/// Redondea `n` bytes hacia arriba a un múltiplo de 16, conservando cero.
/// Se usa para reservar locales; suma 15 y borra los cuatro bits inferiores.
fn align16(n: usize) -> usize {
    if n == 0 {
        0
    } else {
        (n + 15) & !15
    }
}

/// Añade a `s` las instrucciones de la función `f`; `strings` proporciona
/// longitudes de literales y `_main` no se usa. Reserva ocho bytes por variable
/// respecto a rbp y guarda los seis registros de argumentos en posiciones locales.
/// Traduce la pila IR a push/pop y usa profundidades verificadas por flujo para ajustar
/// la pila antes de call. Los retornos saltan
/// a un epílogo que restaura rsp/rbp. Devuelve error ante locales/cadenas ausentes
/// o llamadas de más de seis argumentos. Presupone parámetros válidos en la IR.
/// Usa r10 como temporal volátil y conserva los registros exigidos por la ABI.
fn emit_fn(s: &mut String, f: &FunctionIr, _main: bool, strings: &[Vec<u8>]) -> Result<(), String> {
    let mut vars: Vec<String> = f.params.clone();
    for x in &f.locals {
        if !vars.contains(x) {
            vars.push(x.clone());
        }
    }
    // Cada variable ocupa ocho bytes en un desplazamiento negativo respecto a rbp.
    let mut slots = HashMap::new();
    for (i, n) in vars.iter().enumerate() {
        slots.insert(n.clone(), 8 * (i + 1));
    }
    let mut frame_bytes = vars.len() * 8;
    let mut records = HashMap::new();
    for ins in &f.code {
        if let Instr::TryBegin { record, .. } = ins {
            if !records.contains_key(record) {
                frame_bytes += 32;
                records.insert(record.clone(), frame_bytes);
            }
        }
    }
    let local_size = align16(frame_bytes);
    let (depths, maximum) = crate::verify::verify(f)?;
    // Reserva margen para rutinas nativas antes de tocar un marco demasiado profundo.
    let required = local_size
        .checked_add(maximum.checked_mul(8).ok_or("pila demasiado grande")?)
        .and_then(|n| n.checked_add(512))
        .ok_or("pila demasiado grande")?;
    let end = format!(".L_{}_epilogue", f.name);
    s.push_str(&format!(
        "{}:\n    mov r11, {required}\n    mov rax, rsp\n    sub rax, r11\n    jc pf_stack_error\n    cmp rax, [rip+pf_stack_floor]\n    jb pf_stack_error\n    push rbp\n    mov rbp, rsp\n",
        sym(&f.name)
    ));
    if local_size > 0 {
        s.push_str(&format!("    sub rsp, {local_size}\n"));
    }
    // Guarda los argumentos de los seis registros de llamada en sus posiciones locales.
    let regs = ["rdi", "rsi", "rdx", "rcx", "r8", "r9"];
    for (i, p) in f.params.iter().enumerate() {
        let off = slots[p];
        s.push_str(&format!("    mov QWORD PTR [rbp-{off}], {}\n", regs[i]));
    }
    // La profundidad procede del análisis de flujo, no del orden textual de las ramas.
    for (index, ins) in f.code.iter().enumerate() {
        let Some(mut depth) = depths[index] else {
            continue;
        };
        match ins {
            Instr::TryBegin { record, catch } => {
                let off = records[record];
                s.push_str(&format!("    lea r10, [rbp-{off}]\n    mov rax, [rip+pf_handler]\n    mov [r10], rax\n    mov [r10+8], rsp\n    mov [r10+16], rbp\n    lea rax, [rip+.{catch}]\n    mov [r10+24], rax\n    mov [rip+pf_handler], r10\n"));
            }
            Instr::TryEnd { record } => {
                let off = records[record];
                s.push_str(&format!(
                    "    mov rax, [rbp-{off}]\n    mov [rip+pf_handler], rax\n"
                ));
            }
            Instr::ExceptionValue => s.push_str("    push QWORD PTR [rip+pf_exception]\n"),
            Instr::Throw => s.push_str("    pop rdi\n    jmp pf_throw\n"),

            // Las colecciones usan referencias de 64 bits y rutinas propias sin libc.
            Instr::Collection(op) => {
                let (name, argc) = op.runtime();
                for idx in (0..argc).rev() {
                    s.push_str(&format!("    pop {}\n", regs[idx]));
                    depth = depth.saturating_sub(1);
                }
                let pad = depth % 2 == 1;
                if pad {
                    s.push_str("    sub rsp, 8\n");
                }
                s.push_str(&format!("    call {name}\n"));
                if pad {
                    s.push_str("    add rsp, 8\n");
                }
                s.push_str("    push rax\n");
            }
            Instr::PushText(id) => {
                s.push_str(&format!("    lea rax, [rip+pf_text_{id}]\n    push rax\n"));
            }
            Instr::Scalar(op) => {
                let (name, argc) = op.runtime();
                for idx in (0..argc).rev() {
                    s.push_str(&format!("    pop {}\n", regs[idx]));
                    depth = depth.saturating_sub(1);
                }
                let pad = depth % 2 == 1;
                if pad {
                    s.push_str("    sub rsp, 8\n");
                }
                s.push_str(&format!("    call {name}\n"));
                if pad {
                    s.push_str("    add rsp, 8\n");
                }
                s.push_str("    push rax\n");
            }
            Instr::PushI64(n) => {
                s.push_str(&format!("    mov rax, {n}\n    push rax\n"));
            }
            Instr::Load(n) => {
                let off = *slots
                    .get(n)
                    .ok_or_else(|| format!("generación: variable local desconocida {n}"))?;
                s.push_str(&format!(
                    "    mov rax, QWORD PTR [rbp-{off}]\n    push rax\n"
                ));
            }
            Instr::Store(n) => {
                let off = *slots
                    .get(n)
                    .ok_or_else(|| format!("generación: variable local desconocida {n}"))?;
                s.push_str(&format!(
                    "    pop rax\n    mov QWORD PTR [rbp-{off}], rax\n"
                ));
            }
            // Desapila derecho a r10 e izquierdo a rax; idiv deja cociente en rax y resto en rdx.
            Instr::Add | Instr::Sub | Instr::Mul | Instr::Div | Instr::Mod => {
                s.push_str("    pop r10\n    pop rax\n");
                match ins {
                    Instr::Add => s.push_str("    add rax, r10\n    jo pf_integer_overflow\n"),
                    Instr::Sub => s.push_str("    sub rax, r10\n    jo pf_integer_overflow\n"),
                    Instr::Mul => s.push_str("    imul rax, r10\n    jo pf_integer_overflow\n"),
                    Instr::Div => s.push_str("    test r10, r10\n    jz pf_integer_zero\n    mov r11, -9223372036854775808\n    cmp rax, r11\n    jne 1f\n    cmp r10, -1\n    je pf_integer_overflow\n1:\n    cqo\n    idiv r10\n"),
                    Instr::Mod => s.push_str("    test r10, r10\n    jz pf_integer_zero\n    mov r11, -9223372036854775808\n    cmp rax, r11\n    jne 1f\n    cmp r10, -1\n    je pf_integer_overflow\n1:\n    cqo\n    idiv r10\n    mov rax, rdx\n"),
                    _ => {}
                }
                s.push_str("    push rax\n");
            }
            Instr::Neg => {
                s.push_str("    pop rax\n    neg rax\n    jo pf_integer_overflow\n    push rax\n")
            }
            Instr::Not => s.push_str(
                "    pop rax\n    test rax, rax\n    sete al\n    movzx rax, al\n    push rax\n",
            ),
            // cmp fija banderas y setcc/movzx producen el booleano 0/1.
            Instr::CmpEq
            | Instr::CmpNe
            | Instr::CmpLt
            | Instr::CmpLe
            | Instr::CmpGt
            | Instr::CmpGe => {
                s.push_str("    pop r10\n    pop rax\n    cmp rax, r10\n");
                let cc = match ins {
                    Instr::CmpEq => "sete",
                    Instr::CmpNe => "setne",
                    Instr::CmpLt => "setl",
                    Instr::CmpLe => "setle",
                    Instr::CmpGt => "setg",
                    Instr::CmpGe => "setge",
                    _ => unreachable!(),
                };
                s.push_str(&format!("    {cc} al\n    movzx rax, al\n    push rax\n"));
            }
            Instr::Label(l) => s.push_str(&format!(".{l}:\n")),
            Instr::Jump(l) => s.push_str(&format!("    jmp .{l}\n")),
            Instr::JumpIfFalse(l) => {
                s.push_str("    pop rax\n    test rax, rax\n");
                s.push_str(&format!("    jz .{l}\n"));
            }
            Instr::JumpIfTrue(l) => {
                s.push_str("    pop rax\n    test rax, rax\n");
                s.push_str(&format!("    jnz .{l}\n"));
            }
            // Recupera argumentos en orden inverso y ajusta la pila según depth antes de call.
            Instr::Call { name, argc } => {
                if *argc > 6 {
                    return Err("el generador admite hasta seis argumentos".into());
                }
                for idx in (0..*argc).rev() {
                    s.push_str(&format!("    pop {}\n", regs[idx]));
                    depth = depth.saturating_sub(1);
                }
                let pad = depth % 2 == 1;
                if pad {
                    s.push_str("    sub rsp, 8\n");
                }
                s.push_str(&format!("    call {}\n", sym(name)));
                if pad {
                    s.push_str("    add rsp, 8\n");
                }
                s.push_str("    push rax\n");
            }
            Instr::Pop => {
                s.push_str("    add rsp, 8\n");
            }
            Instr::PrintInt => {
                s.push_str("    pop rdi\n");
                depth = depth.saturating_sub(1);
                let pad = depth % 2 == 1;
                if pad {
                    s.push_str("    sub rsp, 8\n");
                }
                s.push_str("    call pf_print_i64\n");
                if pad {
                    s.push_str("    add rsp, 8\n");
                }
            }
            Instr::PrintBool => {
                s.push_str("    pop rdi\n");
                depth = depth.saturating_sub(1);
                let pad = depth % 2 == 1;
                if pad {
                    s.push_str("    sub rsp, 8\n");
                }
                s.push_str("    call pf_print_bool\n");
                if pad {
                    s.push_str("    add rsp, 8\n");
                }
            }
            // Imprime el literal con write usando su dirección y longitud.
            Instr::PrintStr(id) => {
                let len = strings
                    .get(*id)
                    .ok_or_else(|| format!("generación: falta el literal de texto {id}"))?
                    .len();
                s.push_str(&format!("    lea rsi, [rip+pf_str_{id}]\n    mov rdx, {len}\n    mov rax, 1\n    mov rdi, 1\n    syscall\n"));
            }
            Instr::ReturnValue => {
                s.push_str("    pop rax\n");
                s.push_str(&format!("    jmp {end}\n"));
            }
            Instr::ReturnVoid => {
                s.push_str("    xor rax, rax\n");
                s.push_str(&format!("    jmp {end}\n"));
            }
        }
    }
    // Epílogo común: descarta temporales, restaura rbp y retorna al llamador.
    s.push_str(&format!(
        "{end}:\n    mov rsp, rbp\n    pop rbp\n    ret\n\n"
    ));
    Ok(())
}

/// Rutinas de ejecución incluidas literalmente en el ensamblador generado.
/// pf_print_bool recibe rdi, elige True/False y comparte la salida en .L_bool_write.
/// pf_print_i64 reserva 64 bytes y construye dígitos de atrás adelante.
/// r8b recuerda el signo; .L_i64_abs_ready trata la magnitud y el caso cero.
/// .L_i64_loop divide sin signo entre diez y convierte el resto a ASCII.
/// La división sin signo permite tratar también la magnitud de i64::MIN.
/// .L_i64_sign añade el signo negativo y .L_i64_write imprime y restaura la pila.
/// Para write: rax=1, rdi=1 (salida estándar), rsi=búfer, rdx=longitud.
/// syscall modifica rcx y r11. No se reintentan escrituras parciales.
/// Los comentarios están fuera del literal para conservar exactamente su contenido.
const RUNTIME: &str = r#"
pf_print_literal:
    mov rdx, rsi
    mov rsi, rdi
    mov rax, 1
    mov rdi, 1
    syscall
    ret

# Operaciones comprobadas usadas por el backend LLVM. Permanecen en el runtime
# para conservar exactamente los mismos errores de desbordamiento y división.
pf_llvm_add:
    mov rax, rdi
    add rax, rsi
    jo pf_integer_overflow
    ret
pf_llvm_sub:
    mov rax, rdi
    sub rax, rsi
    jo pf_integer_overflow
    ret
pf_llvm_mul:
    mov rax, rdi
    imul rax, rsi
    jo pf_integer_overflow
    ret
pf_llvm_div:
    test rsi, rsi
    jz pf_integer_zero
    mov rax, rdi
    mov r10, -9223372036854775808
    cmp rax, r10
    jne .L_llvm_div_ready
    cmp rsi, -1
    je pf_integer_overflow
.L_llvm_div_ready:
    cqo
    idiv rsi
    ret
pf_llvm_mod:
    test rsi, rsi
    jz pf_integer_zero
    mov rax, rdi
    mov r10, -9223372036854775808
    cmp rax, r10
    jne .L_llvm_mod_ready
    cmp rsi, -1
    je pf_integer_overflow
.L_llvm_mod_ready:
    cqo
    idiv rsi
    mov rax, rdx
    ret
pf_llvm_neg:
    mov rax, rdi
    neg rax
    jo pf_integer_overflow
    ret

pf_print_bool:
    test rdi, rdi
    jz .L_bool_false
    lea rsi, [rip+pf_true]
    mov rdx, 5
    jmp .L_bool_write
.L_bool_false:
    lea rsi, [rip+pf_false]
    mov rdx, 6
.L_bool_write:
    mov rax, 1
    mov rdi, 1
    syscall
    ret

pf_print_i64:
    push rbp
    mov rbp, rsp
    sub rsp, 64
    mov rax, rdi
    lea rsi, [rbp-1]
    mov BYTE PTR [rsi], 10
    mov rcx, 1
    xor r8d, r8d
    test rax, rax
    jge .L_i64_abs_ready
    mov r8b, 1
    neg rax
.L_i64_abs_ready:
    test rax, rax
    jne .L_i64_loop
    dec rsi
    mov BYTE PTR [rsi], 48
    inc rcx
    jmp .L_i64_sign
.L_i64_loop:
    xor rdx, rdx
    mov r9, 10
    div r9
    add dl, 48
    dec rsi
    mov BYTE PTR [rsi], dl
    inc rcx
    test rax, rax
    jne .L_i64_loop
.L_i64_sign:
    test r8b, r8b
    jz .L_i64_write
    dec rsi
    mov BYTE PTR [rsi], 45
    inc rcx
.L_i64_write:
    mov rax, 1
    mov rdi, 1
    mov rdx, rcx
    syscall
    mov rsp, rbp
    pop rbp
    ret
"#;
