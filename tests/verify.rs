//! Casos de IR inválida que el generador debe rechazar antes de emitir código.
use pyforge::{
    ast::TypeName,
    ir::{FunctionIr, Instr},
    verify::verify,
};

/// Construye una función mínima para aislar errores de flujo de los analizadores.
fn function(code: Vec<Instr>) -> FunctionIr {
    FunctionIr {
        name: "prueba".into(),
        params: vec![],
        locals: vec!["x".into()],
        return_type: TypeName::Int,
        code,
    }
}

/// Las comprobaciones cubren subdesbordamiento de pila y destinos de salto inexistentes.
#[test]
fn pila_y_destinos() {
    assert!(verify(&function(vec![Instr::Pop, Instr::ReturnVoid])).is_err());
    assert!(verify(&function(vec![Instr::Jump("ausente".into())])).is_err());
    assert!(verify(&function(vec![])).is_err());
}

/// La intersección de ramas impide usar una variable inicializada solo en un camino.
#[test]
fn inicializacion_en_ramas() {
    let result = verify(&function(vec![
        Instr::PushI64(1),
        Instr::JumpIfFalse("fin".into()),
        Instr::PushI64(2),
        Instr::Store("x".into()),
        Instr::Label("fin".into()),
        Instr::Load("x".into()),
        Instr::ReturnValue,
    ]));
    assert!(result.unwrap_err().contains("sin inicializar"));
}

/// Un punto de unión no puede recibir cantidades distintas de temporales.
#[test]
fn profundidad_en_ramas() {
    let result = verify(&function(vec![
        Instr::PushI64(1),
        Instr::JumpIfFalse("fin".into()),
        Instr::PushI64(2),
        Instr::Label("fin".into()),
        Instr::ReturnVoid,
    ]));
    assert!(result.is_err());
}

/// Ningún salto ni retorno puede abandonar registros que apunten a marcos ya retirados.
#[test]
fn integridad_de_manejadores() {
    assert!(verify(&function(vec![
        Instr::TryEnd { record: "r".into() },
        Instr::ReturnVoid
    ]))
    .is_err());
    assert!(verify(&function(vec![
        Instr::TryBegin {
            record: "r".into(),
            catch: "error".into()
        },
        Instr::ReturnVoid,
        Instr::Label("error".into()),
        Instr::ReturnVoid,
    ]))
    .is_err());
    assert!(verify(&function(vec![
        Instr::TryBegin {
            record: "r".into(),
            catch: "fin".into()
        },
        Instr::Jump("fin".into()),
        Instr::Label("fin".into()),
        Instr::ReturnVoid,
    ]))
    .is_err());
}
