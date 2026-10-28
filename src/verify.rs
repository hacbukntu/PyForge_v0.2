//! Verificación de flujo de la IR antes de emitir instrucciones de máquina.
//! Comprueba equilibrio de pila, inicialización en todos los caminos y retornos.
use crate::ir::{FunctionIr, Instr};
use std::collections::{HashMap, HashSet, VecDeque};

/// Devuelve la profundidad antes de cada instrucción alcanzable y la profundidad máxima.
/// Al unir ramas exige pilas iguales e intersecta variables definitivamente inicializadas.
/// Reprocesa sucesores cuando cambia esa intersección, también en presencia de bucles.
pub fn verify(f: &FunctionIr) -> Result<(Vec<Option<usize>>, usize), String> {
    let mut labels = HashMap::new();
    for (i, ins) in f.code.iter().enumerate() {
        if let Instr::Label(name) = ins {
            if labels.insert(name, i).is_some() {
                return Err(format!("etiqueta duplicada: {name}"));
            }
        }
    }
    let mut states: Vec<Option<(usize, HashSet<String>, Vec<String>)>> =
        vec![None; f.code.len() + 1];
    states[0] = Some((0, f.params.iter().cloned().collect(), Vec::new()));
    let mut queue = VecDeque::from([0]);
    let mut maximum = 0;
    while let Some(i) = queue.pop_front() {
        if i == f.code.len() {
            return Err(format!("la función {} puede terminar sin retornar", f.name));
        }
        let (depth, mut initialized, mut handlers) = states[i].clone().unwrap();
        let ins = &f.code[i];
        let (take, give) = match ins {
            Instr::PushI64(_) | Instr::PushText(_) | Instr::ExceptionValue => (0, 1),
            Instr::Load(name) => {
                if !initialized.contains(name) {
                    return Err(format!(
                        "variable posiblemente sin inicializar: {name} en {}",
                        f.name
                    ));
                }
                (0, 1)
            }
            Instr::Store(name) => {
                initialized.insert(name.clone());
                (1, 0)
            }
            Instr::Add
            | Instr::Sub
            | Instr::Mul
            | Instr::Div
            | Instr::Mod
            | Instr::CmpEq
            | Instr::CmpNe
            | Instr::CmpLt
            | Instr::CmpLe
            | Instr::CmpGt
            | Instr::CmpGe => (2, 1),
            Instr::Neg | Instr::Not => (1, 1),
            Instr::Call { argc, .. } => (*argc, 1),
            Instr::Collection(op) => (op.runtime().1, 1),
            Instr::Scalar(op) => (op.runtime().1, 1),
            Instr::Throw
            | Instr::Pop
            | Instr::PrintInt
            | Instr::PrintBool
            | Instr::ReturnValue
            | Instr::JumpIfFalse(_)
            | Instr::JumpIfTrue(_) => (1, 0),
            _ => (0, 0),
        };
        if depth < take {
            return Err(format!("pila insuficiente en {}: instrucción {i}", f.name));
        }
        let next_depth = depth - take + give;
        maximum = maximum.max(depth).max(next_depth);
        let mut successors = Vec::new();
        match ins {
            Instr::Throw => {
                if next_depth != 0 {
                    return Err("lanzar con temporales pendientes".into());
                }
            }
            Instr::TryEnd { record } => {
                if handlers.pop().as_ref() != Some(record) {
                    return Err("retiro de un manejador de excepciones que no está activo".into());
                }
                successors.push(i + 1);
            }
            Instr::TryBegin { record, catch } => {
                if handlers.contains(record) {
                    return Err("registro de excepciones activado dos veces".into());
                }
                if depth != 0 {
                    return Err("intentar requiere una pila de evaluación vacía".into());
                }
                // Una excepción puede ocurrir antes de cualquier asignación del cuerpo.
                successors.push(
                    *labels
                        .get(catch)
                        .ok_or_else(|| format!("capturador desconocido: {catch}"))?,
                );
                successors.push(i + 1);
            }
            Instr::ReturnValue | Instr::ReturnVoid => {
                if !handlers.is_empty() {
                    return Err("retorno con manejadores de excepciones pendientes".into());
                }
                if next_depth != 0 {
                    return Err(format!("retorno con temporales pendientes en {}", f.name));
                }
            }
            Instr::Jump(label) | Instr::JumpIfFalse(label) | Instr::JumpIfTrue(label) => {
                successors.push(
                    *labels
                        .get(label)
                        .ok_or_else(|| format!("etiqueta desconocida: {label}"))?,
                );
                if !matches!(ins, Instr::Jump(_)) {
                    successors.push(i + 1);
                }
            }
            _ => successors.push(i + 1),
        }
        for (edge, target) in successors.into_iter().enumerate() {
            // El runtime retira el registro antes de saltar al capturador; solo
            // el sucesor normal entra con un manejador adicional activo.
            let mut next_handlers = handlers.clone();
            if let Instr::TryBegin { record, .. } = ins {
                if edge == 1 {
                    next_handlers.push(record.clone());
                }
            }
            let merged = if let Some((old_depth, old_vars, old_handlers)) = &states[target] {
                if *old_handlers != next_handlers {
                    return Err("manejadores incompatibles al unir ramas".into());
                }
                if *old_depth != next_depth {
                    return Err(format!("pilas incompatibles al unir ramas de {}", f.name));
                }
                old_vars.intersection(&initialized).cloned().collect()
            } else {
                initialized.clone()
            };
            let state = (next_depth, merged, next_handlers);
            if states[target].as_ref() != Some(&state) {
                states[target] = Some(state);
                queue.push_back(target);
            }
        }
    }
    Ok((
        states[..f.code.len()]
            .iter()
            .map(|s| s.as_ref().map(|x| x.0))
            .collect(),
        maximum,
    ))
}
