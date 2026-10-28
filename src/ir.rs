//! Representación intermedia (IR) compartida entre lower, optimizer y codegen.
//! Modela una pila de valores, variables por nombre, llamadas y saltos simbólicos.
//! Todavía no contiene direcciones de máquina ni posiciones concretas de variables.
use crate::ast::TypeName;

#[derive(Debug, Clone)]
/// Instrucciones de pila: PushI64/Load añaden un valor; Store/Pop lo consumen.
/// Las operaciones binarias consumen dos y producen uno. Los saltos condicionales
/// consumen su condición. Call consume argumentos y apila el retorno.
/// PrintInt/Bool consumen un valor; PrintStr referencia datos estáticos por índice.
/// Label/Jump expresan control de flujo sin direcciones de máquina todavía.
pub enum Instr {
    /// Registro de cuatro palabras en el marco local: anterior, rsp, rbp y destino.
    TryBegin {
        record: String,
        catch: String,
    },
    /// Retira el registro antes de una salida normal, retorno o salto de bucle.
    TryEnd {
        record: String,
    },
    /// Apila el mensaje recibido por el capturador; Throw transfiere el control.
    ExceptionValue,
    Throw,
    /// Dirección de descriptor UTF-8 inmutable (longitud en bytes seguida del contenido).
    PushText(usize),
    /// Operación sobre escalares que recibe palabras de 64 bits y devuelve una palabra.
    Scalar(ScalarOp),
    /// Operación nativa de colección, con argumentos y resultado en la pila.
    Collection(CollectionOp),
    PushI64(i64),
    Load(String),
    Store(String),
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    Not,
    CmpEq,
    CmpNe,
    CmpLt,
    CmpLe,
    CmpGt,
    CmpGe,
    Label(String),
    Jump(String),
    JumpIfFalse(String),
    JumpIfTrue(String),
    Call {
        name: String,
        argc: usize,
    },
    Pop,
    PrintInt,
    PrintBool,
    PrintStr(usize),
    ReturnValue,
    ReturnVoid,
}

#[derive(Debug, Clone)]
/// Función intermedia: name identifica el símbolo, params y locals sus variables,
/// return_type el retorno y code las instrucciones en orden.
pub struct FunctionIr {
    pub name: String,
    pub params: Vec<String>,
    pub locals: Vec<String>,
    pub return_type: TypeName,
    pub code: Vec<Instr>,
}

#[derive(Debug, Clone, Default)]
/// IR completa: strings contiene bytes de literales, functions las funciones
/// de usuario y main el bloque superior convertido en función opcional.
pub struct ProgramIr {
    pub strings: Vec<Vec<u8>>,
    pub functions: Vec<FunctionIr>,
    pub main: Option<FunctionIr>,
}

/// Representa `p` como texto para el comando emit-ir.
/// Muestra cadenas con decodificación UTF-8 tolerante, funciones y main al final.
/// Usa Debug para cada instrucción; no ejecuta ni transforma la IR.
pub fn format_ir(p: &ProgramIr) -> String {
    let mut s = String::new();
    for (i, st) in p.strings.iter().enumerate() {
        s.push_str(&format!(".string {i} {:?}\n", String::from_utf8_lossy(st)));
    }
    for f in p.functions.iter().chain(p.main.iter()) {
        s.push_str(&format!(
            "\nfn {}({}) -> {}\n",
            f.name,
            f.params.join(", "),
            f.return_type.as_str()
        ));
        for ins in &f.code {
            s.push_str(&format!("  {:?}\n", ins));
        }
    }
    s
}

/// Operaciones de colecciones compartidas por lower y el generador nativo.
#[derive(Debug, Clone, Copy)]
pub enum CollectionOp {
    TextDictGet,
    TextDictSet,
    SetAdd,
    New,
    Append,
    /// Añade valor y etiqueta a una lista; la etiqueta permite listas mixtas.
    AppendTagged,
    /// Copia una caja dinámica obtenida de otra lista mixta.
    AppendDynamic,
    ListGet,
    ListGetDynamic,
    ListSet,
    ListSetTagged,
    ListSetDynamic,
    DictGet,
    DictSet,
    Length,
    ContainsList,
    ContainsListTagged,
    ContainsListDynamic,
    ContainsDict,
}
impl CollectionOp {
    /// Devuelve símbolo de rutina y número de argumentos que se consumen.
    pub fn runtime(self) -> (&'static str, usize) {
        match self {
            Self::TextDictGet => ("pf_text_dict_get", 2),
            Self::TextDictSet => ("pf_text_dict_set", 3),
            Self::SetAdd => ("pf_set_add", 2),
            Self::New => ("pf_col_new", 0),
            Self::Append => ("pf_list_append", 2),
            Self::AppendTagged => ("pf_list_append_tagged", 3),
            Self::AppendDynamic => ("pf_list_append_dynamic", 2),
            Self::ListGet => ("pf_list_get", 2),
            Self::ListGetDynamic => ("pf_list_get_dynamic", 2),
            Self::ListSet => ("pf_list_set", 3),
            Self::ListSetTagged => ("pf_list_set_tagged", 4),
            Self::ListSetDynamic => ("pf_list_set_dynamic", 3),
            Self::DictGet => ("pf_dict_get", 2),
            Self::DictSet => ("pf_dict_set", 3),
            Self::Length => ("pf_col_length", 1),
            Self::ContainsList => ("pf_std_contains_list", 2),
            Self::ContainsListTagged => ("pf_std_contains_list_tagged", 3),
            Self::ContainsListDynamic => ("pf_std_contains_list_dynamic", 2),
            Self::ContainsDict => ("pf_std_contains_dict", 2),
        }
    }
}

/// Rutinas escalares nativas; las impresiones devuelven cero y el llamador lo descarta.
#[derive(Debug, Clone, Copy)]
pub enum ScalarOp {
    /// Indica si la entrada estándar es un terminal compatible con termios.
    KeyboardIsTerminal,
    /// Lee una tecla sin esperar Enter y restaura termios antes de retornar.
    KeyboardRead,
    /// Llamadas tipadas: convierten errores después de volver de Rust, nunca desenrollan su pila.
    ResourceOpen,
    ResourceBuffer,
    ResourceFromText,
    ResourceRead,
    ResourceWrite,
    ResourceLen,
    ResourceGet,
    ResourceSet,
    ResourceText,
    ResourceClose,

    /// ABI 1 del runtime Rust opcional; recursos opacos, sin auxiliar Python.
    NativeVersion,
    NativeOpen,
    NativeClose,
    NativeBuffer,
    NativeFromText,
    NativeRead,
    NativeWrite,
    NativeLen,
    NativeGet,
    NativeSet,
    NativeUtf8,
    NativeText,

    /// Petición JSON al auxiliar opcional, con permisos y recursos propios.
    IntegrationCall,
    /// Conversiones escalares comprobadas; un fallo de valor lanza una excepción ST.
    TextInt,
    TextFloat,
    FloatInt,
    IntFloat,
    FloatBool,
    /// Milisegundos Linux: reloj indicado por su identificador.
    ClockMillis,
    /// Espera en milisegundos, reanudada tras señales.
    SleepMillis,
    /// Directorio actual UTF-8 del proceso.
    CurrentDirectory,
    /// Consulta de existencia con resultado de syscall comprobable.
    PathExists,
    /// Creación de directorio con permisos sujetos a umask.
    MakeDirectory,
    /// Eliminación de un archivo, sin recursión ni shell.
    RemoveFile,
    /// Raíz cuadrada IEEE-754 de hardware.
    SquareRoot,
    /// Codifica un valor escalar Unicode en una cadena UTF-8.
    UnicodeText,
    /// Primer escalar Unicode de una cadena, o -1 si está vacía.
    UnicodeCode,
    /// Imprime un mensaje sin salto de línea y después lee una línea.
    InputPrompt,
    Input,
    ReadFile,
    WriteFile,
    TextIndex,
    TextFind,
    TextReplace,
    IntText,
    FloatText,
    BoolText,
    NullText,
    /// Operaciones sobre la caja estable `{etiqueta, valor}` de lista[mixto].
    DynamicType,
    DynamicText,
    DynamicInt,
    DynamicFloat,
    DynamicBool,
    DynamicPrint,
    TextPrint,
    TextConcat,
    TextEq,
    TextLength,
    FloatPrint,
    FloatAdd,
    FloatSub,
    FloatMul,
    FloatDiv,
    FloatNeg,
    FloatEq,
    FloatNe,
    FloatLt,
    FloatLe,
    FloatGt,
    FloatGe,
}
impl ScalarOp {
    /// Símbolo ensamblador y cantidad de argumentos de la operación.
    pub fn runtime(self) -> (&'static str, usize) {
        match self {
            Self::KeyboardIsTerminal => ("pf_keyboard_is_terminal", 0),
            Self::KeyboardRead => ("pf_keyboard_read", 1),
            Self::ResourceOpen => ("pf_resource_open", 2),
            Self::ResourceBuffer => ("pf_resource_buffer", 1),
            Self::ResourceFromText => ("pf_resource_from_text", 1),
            Self::ResourceRead => ("pf_resource_read", 2),
            Self::ResourceWrite => ("pf_resource_write", 2),
            Self::ResourceLen => ("pf_resource_len", 1),
            Self::ResourceGet => ("pf_resource_get", 2),
            Self::ResourceSet => ("pf_resource_set", 3),
            Self::ResourceText => ("pf_resource_text", 1),
            Self::ResourceClose => ("pf_resource_close", 1),
            Self::NativeVersion => ("pf_native_version", 0),
            Self::NativeOpen => ("pf_native_open", 2),
            Self::NativeClose => ("pf_native_close", 1),
            Self::NativeBuffer => ("pf_native_buffer", 1),
            Self::NativeFromText => ("pf_native_from_text", 1),
            Self::NativeRead => ("pf_native_read", 2),
            Self::NativeWrite => ("pf_native_write", 2),
            Self::NativeLen => ("pf_native_len", 1),
            Self::NativeGet => ("pf_native_get", 2),
            Self::NativeSet => ("pf_native_set", 3),
            Self::NativeUtf8 => ("pf_native_utf8", 1),
            Self::NativeText => ("pf_native_text", 1),
            Self::IntegrationCall => ("pf_bridge_call", 1),
            Self::TextInt => ("pf_convert_text_int", 1),
            Self::TextFloat => ("pf_convert_text_float", 1),
            Self::FloatInt => ("pf_convert_float_int", 1),
            Self::IntFloat => ("pf_convert_int_float", 1),
            Self::FloatBool => ("pf_convert_float_bool", 1),
            Self::ClockMillis => ("pf_std_clock", 1),
            Self::SleepMillis => ("pf_std_sleep", 1),
            Self::CurrentDirectory => ("pf_std_cwd", 0),
            Self::PathExists => ("pf_std_exists", 1),
            Self::MakeDirectory => ("pf_std_mkdir", 1),
            Self::RemoveFile => ("pf_std_unlink", 1),
            Self::SquareRoot => ("pf_std_sqrt", 1),
            Self::UnicodeCode => ("pf_std_codepoint", 1),
            Self::UnicodeText => ("pf_std_unicode", 1),
            Self::InputPrompt => ("pf_input_prompt", 1),
            Self::Input => ("pf_input", 0),
            Self::ReadFile => ("pf_read_file", 1),
            Self::WriteFile => ("pf_write_file", 2),
            Self::TextIndex => ("pf_text_index", 2),
            Self::TextFind => ("pf_text_find", 2),
            Self::TextReplace => ("pf_text_replace", 3),
            Self::IntText => ("pf_int_text", 1),
            Self::FloatText => ("pf_float_text", 1),
            Self::BoolText => ("pf_bool_text", 1),
            Self::NullText => ("pf_null_text", 1),
            Self::DynamicType => ("pf_dynamic_type", 1),
            Self::DynamicText => ("pf_dynamic_text", 1),
            Self::DynamicInt => ("pf_dynamic_int", 1),
            Self::DynamicFloat => ("pf_dynamic_float", 1),
            Self::DynamicBool => ("pf_dynamic_bool", 1),
            Self::DynamicPrint => ("pf_dynamic_print", 1),
            Self::TextPrint => ("pf_text_print", 1),
            Self::TextConcat => ("pf_text_concat", 2),
            Self::TextEq => ("pf_text_eq", 2),
            Self::TextLength => ("pf_text_length", 1),
            Self::FloatPrint => ("pf_float_print", 1),
            Self::FloatNeg => ("pf_float_neg", 1),
            Self::FloatAdd => ("pf_float_add", 2),
            Self::FloatSub => ("pf_float_sub", 2),
            Self::FloatMul => ("pf_float_mul", 2),
            Self::FloatDiv => ("pf_float_div", 2),
            Self::FloatEq => ("pf_float_eq", 2),
            Self::FloatNe => ("pf_float_ne", 2),
            Self::FloatLt => ("pf_float_lt", 2),
            Self::FloatLe => ("pf_float_le", 2),
            Self::FloatGt => ("pf_float_gt", 2),
            Self::FloatGe => ("pf_float_ge", 2),
        }
    }
}
