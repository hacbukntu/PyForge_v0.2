//! Biblioteca estándar incrustada en pyforgec: no busca archivos en el equipo del usuario.
//! Las funciones ST reutilizan el analizador habitual. Solo las primitivas que necesitan
//! instrucciones de CPU o llamadas Linux reciben un AST TextOp construido aquí.
use crate::ast::{Expr, FunctionDef, Param, Program, Stmt, TypeName, Visibility};
use crate::ir::ScalarOp;

/// Firma única de una primitiva: semántica y lowering consultan esta misma tabla.
pub struct Native {
    /// Módulo propietario, registrado en source.
    pub module: &'static str,
    /// Función auxiliar privada que invocan las fuentes ST.
    pub name: &'static str,
    /// Tipos de los parámetros, en el orden del convenio de llamadas.
    pub args: &'static [TypeName],
    /// Tipo estático de la palabra devuelta por el runtime.
    pub result: TypeName,
    /// Operación IR compartida con verificación y generación de ensamblador.
    pub op: ScalarOp,
}
use TypeName::{Float, Int, NativeBuffer, NativeFile, Str};
/// Los nombres aux_ son privados del módulo estándar, no palabras reservadas nuevas.
pub const NATIVES: &[Native] = &[
    Native {
        module: "motor3d",
        name: "aux_raiz",
        args: &[Float],
        result: Float,
        op: ScalarOp::SquareRoot,
    },
    Native {
        module: "teclado",
        name: "aux_es_terminal",
        args: &[],
        result: Int,
        op: ScalarOp::KeyboardIsTerminal,
    },
    Native {
        module: "teclado",
        name: "aux_leer",
        args: &[Int],
        result: Int,
        op: ScalarOp::KeyboardRead,
    },
    Native {
        module: "recursos",
        name: "aux_abrir",
        args: &[Str, Int],
        result: NativeFile,
        op: ScalarOp::ResourceOpen,
    },
    Native {
        module: "recursos",
        name: "aux_buffer",
        args: &[Int],
        result: NativeBuffer,
        op: ScalarOp::ResourceBuffer,
    },
    Native {
        module: "recursos",
        name: "aux_desde_texto",
        args: &[Str],
        result: NativeBuffer,
        op: ScalarOp::ResourceFromText,
    },
    Native {
        module: "recursos",
        name: "aux_leer",
        args: &[NativeFile, Int],
        result: NativeBuffer,
        op: ScalarOp::ResourceRead,
    },
    Native {
        module: "recursos",
        name: "aux_escribir",
        args: &[NativeFile, NativeBuffer],
        result: Int,
        op: ScalarOp::ResourceWrite,
    },
    Native {
        module: "recursos",
        name: "aux_tamano",
        args: &[NativeBuffer],
        result: Int,
        op: ScalarOp::ResourceLen,
    },
    Native {
        module: "recursos",
        name: "aux_obtener",
        args: &[NativeBuffer, Int],
        result: Int,
        op: ScalarOp::ResourceGet,
    },
    Native {
        module: "recursos",
        name: "aux_poner",
        args: &[NativeBuffer, Int, Int],
        result: Int,
        op: ScalarOp::ResourceSet,
    },
    Native {
        module: "recursos",
        name: "aux_texto",
        args: &[NativeBuffer],
        result: Str,
        op: ScalarOp::ResourceText,
    },
    Native {
        module: "recursos",
        name: "aux_cerrar_archivo",
        args: &[NativeFile],
        result: Int,
        op: ScalarOp::ResourceClose,
    },
    Native {
        module: "recursos",
        name: "aux_cerrar_buffer",
        args: &[NativeBuffer],
        result: Int,
        op: ScalarOp::ResourceClose,
    },
    Native {
        module: "nativo",
        name: "aux_version",
        args: &[],
        result: Int,
        op: ScalarOp::NativeVersion,
    },
    Native {
        module: "nativo",
        name: "aux_abrir",
        args: &[Str, Int],
        result: Int,
        op: ScalarOp::NativeOpen,
    },
    Native {
        module: "nativo",
        name: "aux_cerrar",
        args: &[Int],
        result: Int,
        op: ScalarOp::NativeClose,
    },
    Native {
        module: "nativo",
        name: "aux_buffer",
        args: &[Int],
        result: Int,
        op: ScalarOp::NativeBuffer,
    },
    Native {
        module: "nativo",
        name: "aux_desde_texto",
        args: &[Str],
        result: Int,
        op: ScalarOp::NativeFromText,
    },
    Native {
        module: "nativo",
        name: "aux_leer",
        args: &[Int, Int],
        result: Int,
        op: ScalarOp::NativeRead,
    },
    Native {
        module: "nativo",
        name: "aux_escribir",
        args: &[Int, Int],
        result: Int,
        op: ScalarOp::NativeWrite,
    },
    Native {
        module: "nativo",
        name: "aux_longitud",
        args: &[Int],
        result: Int,
        op: ScalarOp::NativeLen,
    },
    Native {
        module: "nativo",
        name: "aux_obtener",
        args: &[Int, Int],
        result: Int,
        op: ScalarOp::NativeGet,
    },
    Native {
        module: "nativo",
        name: "aux_poner",
        args: &[Int, Int, Int],
        result: Int,
        op: ScalarOp::NativeSet,
    },
    Native {
        module: "nativo",
        name: "aux_utf8",
        args: &[Int],
        result: Int,
        op: ScalarOp::NativeUtf8,
    },
    Native {
        module: "nativo",
        name: "aux_texto",
        args: &[Int],
        result: Str,
        op: ScalarOp::NativeText,
    },
    Native {
        module: "integraciones",
        name: "aux_peticion",
        args: &[Str],
        result: Str,
        op: ScalarOp::IntegrationCall,
    },
    Native {
        module: "json",
        name: "aux_codigo",
        args: &[Str],
        result: Int,
        op: ScalarOp::UnicodeCode,
    },
    Native {
        module: "time",
        name: "aux_reloj",
        args: &[Int],
        result: Int,
        op: ScalarOp::ClockMillis,
    },
    Native {
        module: "time",
        name: "aux_dormir",
        args: &[Int],
        result: Int,
        op: ScalarOp::SleepMillis,
    },
    Native {
        module: "os",
        name: "aux_directorio",
        args: &[],
        result: Str,
        op: ScalarOp::CurrentDirectory,
    },
    Native {
        module: "os",
        name: "aux_existe",
        args: &[Str],
        result: Int,
        op: ScalarOp::PathExists,
    },
    Native {
        module: "os",
        name: "aux_crear",
        args: &[Str],
        result: Int,
        op: ScalarOp::MakeDirectory,
    },
    Native {
        module: "os",
        name: "aux_borrar",
        args: &[Str],
        result: Int,
        op: ScalarOp::RemoveFile,
    },
    Native {
        module: "math",
        name: "aux_raiz",
        args: &[Float],
        result: Float,
        op: ScalarOp::SquareRoot,
    },
    Native {
        module: "json",
        name: "aux_caracter",
        args: &[Int],
        result: Str,
        op: ScalarOp::UnicodeText,
    },
];

/// Identifica intrínsecos creados por el compilador; el lexer impide escribir __ en nombres.
pub fn native(name: &str) -> Option<&'static Native> {
    NATIVES
        .iter()
        .find(|n| name == format!("__std_{}_{}", n.module, n.name))
}

/// Devuelve las fuentes compiladas con include_str!: editar estos archivos requiere recompilar.
fn source(name: &str) -> Option<&'static str> {
    Some(match name {
        "archivos" => include_str!("../stdlib/archivos.st"),
        "db" => include_str!("../stdlib/db.st"),
        "documentos" => include_str!("../stdlib/documentos.st"),
        "red" => include_str!("../stdlib/red.st"),
        "cifrado" => include_str!("../stdlib/cifrado.st"),
        "comandos" => include_str!("../stdlib/comandos.st"),
        "vpn" => include_str!("../stdlib/vpn.st"),
        "sistema" => include_str!("../stdlib/sistema.st"),
        "sistema.archivos" => include_str!("../stdlib/archivos.st"),
        "sistema.comandos" => include_str!("../stdlib/comandos.st"),
        "red.tcp" => include_str!("../stdlib/red.st"),
        "red.udp" => include_str!("../stdlib/red.st"),
        "red.socket" => include_str!("../stdlib/red.st"),
        "red.ssl" => include_str!("../stdlib/red.st"),
        "red.ssh" => include_str!("../stdlib/red.st"),
        "red.ftp" => include_str!("../stdlib/red.st"),
        "red.sftp" => include_str!("../stdlib/red.st"),
        "red.vpn" => include_str!("../stdlib/vpn.st"),
        "red.vpn.diagnostico" => include_str!("../stdlib/vpn.st"),
        "gui" => include_str!("../stdlib/gui.st"),
        "integraciones" => include_str!("../stdlib/integraciones.st"),
        "juego" => include_str!("../stdlib/juego.st"),
        "motor" => include_str!("../stdlib/juego.st"),
        "motor2d" => include_str!("../stdlib/juego.st"),
        "motor3d" => include_str!("../stdlib/motor3d.st"),
        "editor3d" => include_str!("../stdlib/editor3d.st"),
        "time" => include_str!("../stdlib/time.st"),
        "os" => include_str!("../stdlib/os.st"),
        "math" => include_str!("../stdlib/math.st"),
        "io" => include_str!("../stdlib/io.st"),
        "nativo" => include_str!("../stdlib/nativo.st"),
        "recursos" => include_str!("../stdlib/recursos.st"),
        "teclado" => include_str!("../stdlib/teclado.st"),
        "json" => include_str!("../stdlib/json.st"),
        _ => return None,
    })
}

/// Solo los nombres exactos sin extensión seleccionan la biblioteca; math.st sigue siendo local.
pub fn contains(name: &str) -> bool {
    source(name).is_some()
}

/// Construye el módulo y sus puentes tipados, que después se verifican igual que código de usuario.
pub fn load(name: &str) -> Result<(Program, Vec<super::modules::Import>), String> {
    let (mut program, imports) =
        super::parser::parse_module(source(name).ok_or("módulo estándar desconocido")?)?;
    for n in NATIVES.iter().filter(|n| n.module == name) {
        program.functions.push(FunctionDef {
            name: n.name.into(),
            params: n
                .args
                .iter()
                .enumerate()
                .map(|(i, ty)| Param {
                    name: format!("arg{i}"),
                    ty: *ty,
                })
                .collect(),
            ret: n.result,
            body: vec![Stmt::Return(Some(Expr::TextOp {
                name: format!("__std_{}_{}", n.module, n.name),
                args: (0..n.args.len())
                    .map(|i| Expr::Var(format!("arg{i}")))
                    .collect(),
            }))],
            visibility: Visibility::Public,
        });
    }
    Ok((program, imports))
}
