//! Las clases se expanden a funciones nativas y registros de campos fijos.
//! No hay etiquetas dinámicas ni reinterpretaciones accesibles desde el lenguaje.
use crate::ast::*;
use crate::semantic::{infer_expr, FnSig};
use std::collections::HashMap;

/// Nombres privados que no pueden escribirse como identificadores ST.
pub fn symbol(id: usize, kind: &str, name: &str) -> String {
    format!("__obj_{id}_{kind}_{name}")
}

/// Añade constructor y accesores tipados. Todos los campos se inicializan antes
/// de publicar el objeto; su orden determina los argumentos del constructor.
pub fn define(name: String, id: usize, fields: Vec<Param>) -> Vec<FunctionDef> {
    let ty = TypeName::Object(id);
    let mut functions = vec![FunctionDef {
        name,
        params: fields.clone(),
        ret: ty,
        body: vec![Stmt::Return(Some(Expr::Record {
            id,
            values: fields.iter().map(|f| Expr::Var(f.name.clone())).collect(),
        }))],
        visibility: Visibility::Public,
    }];
    for (index, field) in fields.into_iter().enumerate() {
        let receiver = Param {
            name: "yo".into(),
            ty,
        };
        functions.push(FunctionDef {
            name: symbol(id, "get", &field.name),
            params: vec![receiver.clone()],
            ret: field.ty,
            body: vec![Stmt::Return(Some(Expr::RecordSlot {
                object: Box::new(Expr::Var("yo".into())),
                index,
                ty: field.ty,
            }))],
            visibility: Visibility::Private,
        });
        functions.push(FunctionDef {
            name: symbol(id, "set", &field.name),
            params: vec![
                receiver,
                Param {
                    name: "valor".into(),
                    ty: field.ty,
                },
            ],
            ret: TypeName::None,
            body: vec![Stmt::SetRecordSlot {
                object: Expr::Var("yo".into()),
                index,
                value: Expr::Var("valor".into()),
            }],
            visibility: Visibility::Private,
        });
    }
    functions
}

/// Convierte un miembro en llamada estática y antepone el receptor exactamente
/// una vez. Semántica y lower comparten esta resolución para no divergir.
pub fn member_call(
    object: &Expr,
    kind: &str,
    name: &str,
    args: &[Expr],
    vars: &HashMap<String, TypeName>,
    fns: &HashMap<String, FnSig>,
) -> Result<Expr, String> {
    let ty = infer_expr(object, vars, fns)?;

    if kind == "method" && name == "append" && (ty.is_list() || ty == TypeName::Set) {
        if args.len() != 1 {
            return Err("append requiere exactamente un argumento".into());
        }
        return Ok(Expr::Append {
            collection: Box::new(object.clone()),
            value: Box::new(args[0].clone()),
        });
    }

    let TypeName::Object(id) = ty else {
        return Err(format!("el miembro {name} requiere un objeto"));
    };
    let target = symbol(id, kind, name);
    if !fns.contains_key(&target) {
        return Err(format!(
            "miembro desconocido: {name} ({} de objeto)",
            if kind == "method" { "método" } else { "campo" }
        ));
    }
    let mut values = vec![object.clone()];
    values.extend_from_slice(args);
    Ok(Expr::Call {
        name: target,
        args: values,
    })
}
