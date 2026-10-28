//! Simplificación local de la IR producida por lower.
//! driver activa esta etapa según opt, antes de generar ensamblador.
//! La optimización conserva las estructuras del programa y reemplaza listas de instrucciones.
use crate::ir::*;

/// Reemplaza el código de cada función y de main en `program` usando fold.
/// Modifica la IR recibida, pero conserva firmas, locales y tabla de cadenas.
pub fn optimize(program: &mut ProgramIr) {
    for f in &mut program.functions {
        f.code = fold(&f.code);
    }
    if let Some(m) = &mut program.main {
        m.code = fold(&m.code);
    }
}
/// Recorre `input` una vez y devuelve una lista simplificada.
/// Dos PushI64 seguidos de una operación reconocida se sustituyen por el resultado;
/// PushI64 seguido de Pop se elimina. Usa aritmética comprobada y conserva las operaciones que deben fallar en ejecución.
/// No reitera sobre la salida: algunas expresiones constantes quedan sin reducir.
/// El desbordamiento de división constante puede diferir del idiv generado.
fn fold(input: &[Instr]) -> Vec<Instr> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < input.len() {
        if i + 2 < input.len() {
            if let (Instr::PushI64(a), Instr::PushI64(b), op) =
                (&input[i], &input[i + 1], &input[i + 2])
            {
                let v = match op {
                    Instr::Add => a.checked_add(*b),
                    Instr::Sub => a.checked_sub(*b),
                    Instr::Mul => a.checked_mul(*b),
                    Instr::Div if *b != 0 => a.checked_div(*b),
                    Instr::Mod if *b != 0 => a.checked_rem(*b),
                    Instr::CmpEq => Some((a == b) as i64),
                    Instr::CmpNe => Some((a != b) as i64),
                    Instr::CmpLt => Some((a < b) as i64),
                    Instr::CmpLe => Some((a <= b) as i64),
                    Instr::CmpGt => Some((a > b) as i64),
                    Instr::CmpGe => Some((a >= b) as i64),
                    _ => None,
                };
                if let Some(v) = v {
                    out.push(Instr::PushI64(v));
                    i += 3;
                    continue;
                }
            }
        }
        if i + 1 < input.len() {
            if matches!(&input[i], Instr::PushI64(_)) && matches!(&input[i + 1], Instr::Pop) {
                i += 2;
                continue;
            }
        }
        out.push(input[i].clone());
        i += 1;
    }
    out
}
