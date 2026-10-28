//! Árbol de sintaxis abstracta (AST): describe el programa sin instrucciones de máquina.
//! parser construye estos nodos; semantic comprueba sus tipos y lower los convierte a IR.
//! Box permite nodos recursivos de tamaño conocido; Vec conserva el orden de los elementos.
/// Tipo escalar de los elementos: evita reinterpretar referencias como enteros.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementType {
    Integer,
    Float,
    Boolean,
    Text,
    Null,
    /// Valor etiquetado en ejecución; se usa exclusivamente dentro de `lista[mixto]`.
    Dynamic,
    /// Tipo compuesto codificado en grupos de cuatro bits; profundidad acotada.
    Nested(u64),
}
impl ElementType {
    /// Convierte un escalar del lenguaje a tipo de elemento; admite colecciones con un máximo de quince niveles.
    pub fn from_type(ty: TypeName) -> Option<Self> {
        Some(match ty {
            TypeName::Int => Self::Integer,
            TypeName::Float => Self::Float,
            TypeName::Bool => Self::Boolean,
            TypeName::Str => Self::Text,
            TypeName::Null => Self::Null,
            TypeName::Dynamic => Self::Dynamic,
            t if t.is_collection() => Self::Nested(t.encode()?),
            _ => return None,
        })
    }
    /// Recupera el tipo estático usado al leer una posición.
    pub fn ty(self) -> TypeName {
        match self {
            Self::Integer => TypeName::Int,
            Self::Float => TypeName::Float,
            Self::Boolean => TypeName::Bool,
            Self::Text => TypeName::Str,
            Self::Null => TypeName::Null,
            Self::Dynamic => TypeName::Dynamic,
            Self::Nested(code) => TypeName::decode(code),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Tipos representables: Int usa i64, Float usa f64, Bool usa 0/1 y Str una referencia UTF-8,
/// None ausencia de retorno y Unknown un tipo indeterminado.
pub enum TypeName {
    /// Recursos de préstamo léxico; solo usar adquiere propiedad y genera su cierre.
    NativeBuffer,
    NativeFile,
    /// Identidad nominal de clase; nunca se confunde con una colección o un entero.
    Object(usize),
    /// Colecciones homogéneas; listas y diccionarios sin parámetro conservan entero.
    ListOf(ElementType),
    DictOf(ElementType),
    /// Valor etiquetado extraído de una lista heterogénea.
    /// No puede declararse solo: nace al indexar `lista[mixto]`.
    Dynamic,
    Int,
    /// Número IEEE-754 de doble precisión, transportado en una palabra de 64 bits.
    Float,
    /// Ausencia de valor almacenable, distinta de una función sin retorno.
    Null,
    /// Secuencia inmutable de enteros.
    Tuple,
    /// Colección mutable de enteros únicos.
    Set,
    Bool,
    Str,
    None,
    Unknown,
    /// Lista mutable de enteros almacenada por referencia.
    List,
    /// Diccionario mutable de claves de texto calculadas en ejecución y valores enteros.
    Dict,
}

impl TypeName {
    /// No se copian, convierten a enteros ni almacenan en colecciones.
    pub fn is_resource(self) -> bool {
        matches!(self, Self::NativeBuffer | Self::NativeFile)
    }

    /// Codifica el árbol homogéneo sin punteros ni registros globales de tipos.
    /// Cada contenedor consume cuatro bits; el desplazamiento se comprueba antes.
    fn encode(self) -> Option<u64> {
        Some(match self {
            Self::Int => 1,
            Self::Float => 2,
            Self::Bool => 3,
            Self::Str => 4,
            Self::Null => 5,
            Self::Dynamic => 10,
            Self::Tuple => 8,
            Self::Set => 9,
            t if t.is_list() || t.is_dict() => {
                let inner = t.element()?.encode()?;
                if inner > (u64::MAX >> 4) {
                    return None;
                }
                (inner << 4) | if t.is_list() { 6 } else { 7 }
            }
            _ => return None,
        })
    }
    /// Inversa privada: solo recibe códigos producidos por encode.
    fn decode(code: u64) -> Self {
        match code & 15 {
            1 => Self::Int,
            2 => Self::Float,
            3 => Self::Bool,
            4 => Self::Str,
            5 => Self::Null,
            8 => Self::Tuple,
            9 => Self::Set,
            10 => Self::Dynamic,
            6 | 7 => Self::collection(
                if code & 15 == 6 {
                    Self::List
                } else {
                    Self::Dict
                },
                Self::decode(code >> 4),
            )
            .unwrap(),
            _ => unreachable!("código interno de tipo inválido"),
        }
    }
    /// Indica qué valor puede almacenarse y recuperarse sin cambiar su representación.
    pub fn element(self) -> Option<TypeName> {
        match self {
            Self::ListOf(t) | Self::DictOf(t) => Some(t.ty()),
            Self::List | Self::Dict | Self::Tuple | Self::Set => Some(Self::Int),
            _ => None,
        }
    }
    /// Normaliza entero al tipo histórico para conservar las firmas de programas anteriores.
    pub fn collection(base: Self, element: Self) -> Option<Self> {
        let t = ElementType::from_type(element)?;
        Some(match base {
            Self::List => {
                if t == ElementType::Integer {
                    Self::List
                } else {
                    Self::ListOf(t)
                }
            }
            Self::Dict => {
                if t == ElementType::Dynamic {
                    return None;
                }
                if t == ElementType::Integer {
                    Self::Dict
                } else {
                    Self::DictOf(t)
                }
            }
            _ => return None,
        })
    }
    /// Clasificadores compartidos por análisis y conversión a IR.
    pub fn is_list(self) -> bool {
        matches!(self, Self::List | Self::ListOf(_))
    }
    pub fn is_dict(self) -> bool {
        matches!(self, Self::Dict | Self::DictOf(_))
    }
    pub fn is_collection(self) -> bool {
        self.element().is_some()
    }

    /// Convierte `s`, sin espacios exteriores, a int, bool, str o None.
    /// Devuelve Some(TypeName) si lo reconoce y None en caso contrario.
    /// El parser lo usa para anotaciones; las restricciones de uso se comprueban después.
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim() {
            "int" => Some(Self::Int),
            "float" => Some(Self::Float),
            "bool" => Some(Self::Bool),
            "str" => Some(Self::Str),
            "None" => Some(Self::None),
            _ => None,
        }
    }

    /// Nombre público de tipo(valor), en español y sin modificar diagnósticos anteriores.
    /// Las colecciones históricas sin parámetro almacenan enteros; las clases son objetos.
    pub fn st_name(self) -> String {
        if self.is_list() || self.is_dict() {
            return format!(
                "{}[{}]",
                if self.is_list() {
                    "lista"
                } else {
                    "diccionario"
                },
                self.element().unwrap().st_name()
            );
        }
        match self {
            Self::Int => "entero",
            Self::Bool => "booleano",
            Self::Str => "cadena",
            Self::List => "lista[entero]",
            Self::Dict => "diccionario[entero]",
            Self::None => "nada",
            _ => self.as_str(),
        }
        .to_owned()
    }

    /// Nombre usado en diagnósticos e IR; conserva las grafías históricas.
    /// La cadena es estática y no requiere asignación; Unknown se muestra como desconocido.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ListOf(ElementType::Integer) => "lista[entero]",
            Self::ListOf(ElementType::Float) => "lista[flotante]",
            Self::ListOf(ElementType::Boolean) => "lista[booleano]",
            Self::ListOf(ElementType::Text) => "lista[cadena]",
            Self::ListOf(ElementType::Null) => "lista[nulo]",
            Self::ListOf(ElementType::Dynamic) => "lista[mixto]",
            Self::DictOf(ElementType::Integer) => "diccionario[entero]",
            Self::DictOf(ElementType::Float) => "diccionario[flotante]",
            Self::DictOf(ElementType::Boolean) => "diccionario[booleano]",
            Self::DictOf(ElementType::Text) => "diccionario[cadena]",
            Self::DictOf(ElementType::Null) => "diccionario[nulo]",
            Self::DictOf(ElementType::Dynamic) => "diccionario[mixto]",
            Self::ListOf(ElementType::Nested(_)) => "lista anidada",
            Self::DictOf(ElementType::Nested(_)) => "diccionario anidado",
            Self::NativeBuffer => "buffer_nativo",
            Self::NativeFile => "archivo_nativo",
            Self::Object(_) => "objeto",
            Self::Dynamic => "mixto",
            Self::Int => "int",
            Self::Float => "flotante",
            Self::Null => "nulo",
            Self::Tuple => "tupla",
            Self::Set => "conjunto",
            Self::Bool => "bool",
            Self::Str => "str",
            Self::None => "None",
            Self::Unknown => "desconocido",
            Self::List => "lista",
            Self::Dict => "diccionario",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Operadores binarios: aritmética, comparación y lógica entre dos expresiones.
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    /// Identidad de referencias del mismo tipo, o identidad del valor nulo.
    Is,
    And,
    Or,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Operadores unarios: Neg cambia el signo, Pos lo conserva y Not niega la verdad.
pub enum UnaryOp {
    Neg,
    Pos,
    Not,
}

#[derive(Debug, Clone)]
/// Expresión del AST: literales, variables por nombre, operaciones o llamadas.
/// Unary/Binary guardan operandos en Box; Call conserva argumentos ordenados.
pub enum Expr {
    /// Campos y métodos se resuelven por el tipo estático del receptor.
    Member {
        object: Box<Expr>,
        name: String,
    },
    MethodCall {
        object: Box<Expr>,
        name: String,
        args: Vec<Expr>,
    },
    /// Comprueba que una llamada construye un objeto; no ejecuta intérpretes.
    New(Box<Expr>),
    /// Nodos internos generados por la declaración de clase, inaccesibles desde ST.
    Record {
        id: usize,
        values: Vec<Expr>,
    },
    RecordSlot {
        object: Box<Expr>,
        index: usize,
        ty: TypeName,
    },
    /// Operación textual incorporada: buscar, reemplazar o conversión explícita.
    TextOp {
        name: String,
        args: Vec<Expr>,
    },
    /// Literal nulo y constructores de colecciones de enteros.
    Null,
    Tuple(Vec<Expr>),
    Set(Vec<Expr>),
    /// Funciones incorporadas de colecciones, separadas de nombres de usuario.
    Length(Box<Expr>),
    Append {
        collection: Box<Expr>,
        value: Box<Expr>,
    },
    /// Literal de lista, evaluado de izquierda a derecha.
    List(Vec<Expr>),
    /// Pares de expresiones clave/valor; una clave repetida reemplaza su valor.
    Dict(Vec<(Expr, Expr)>),
    /// Acceso por índice entero o clave literal.
    Index {
        collection: Box<Expr>,
        key: Box<Expr>,
    },
    /// Literal flotante de ST; el analizador Python conserva su gramática.
    Float(f64),
    Int(i64),
    Bool(bool),
    Str(String),
    Var(String),
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Operación de asignación aumentada: suma, resta, multiplicación, división o resto.
pub enum AugOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

/// Visibilidad de una declaración en el límite de un módulo ST.
///
/// Los nombres privados solo pueden resolverse desde el archivo que los declara.
/// La visibilidad no cambia la representación del valor ni desactiva las
/// comprobaciones de tipos y de inicialización.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
}

/// Distingue una declaración de una escritura posterior.
///
/// ST usa `Declare` para `variable nombre = valor` y `Reassign` para
/// `nombre = valor`. El frontend Python conserva `DefineOrAssign` porque en
/// Python una asignación crea el nombre cuando todavía no existe.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    Declare,
    Reassign,
    DefineOrAssign,
}

#[derive(Debug, Clone)]
/// Sentencia del AST: asignación, expresión suelta, retorno, condición o bucle.
/// Assign lleva anotación opcional; Return puede carecer de valor. If guarda
/// pares (condición, cuerpo) y un else separado. ForRange guarda inicio y límite.
/// Break y Continue alteran el bucle interior; Pass no hace nada.
pub enum Stmt {
    /// Adquisición única y cierre al abandonar el ámbito, incluso por excepción o retorno.
    Using {
        name: String,
        value: Expr,
        body: Vec<Stmt>,
    },
    /// Escritura de campo público; su función auxiliar comprueba el tipo del valor.
    SetMember {
        object: Expr,
        name: String,
        value: Expr,
    },
    /// Escritura interna de posición, solo generada por un setter tipado.
    SetRecordSlot {
        object: Expr,
        index: usize,
        value: Expr,
    },
    /// Excepción explícita con mensaje UTF-8; puede atravesar llamadas nativas.
    Throw(Expr),
    /// El capturador recibe una cadena. La limpieza se ejecuta en todas las salidas.
    Try {
        body: Vec<Stmt>,
        catch: Option<(String, Vec<Stmt>)>,
        finally: Vec<Stmt>,
    },
    /// Selección por literales escalares: evalúa el valor una sola vez.
    Match {
        value: Expr,
        cases: Vec<(Expr, Vec<Stmt>)>,
        otherwise: Vec<Stmt>,
    },
    /// Modifica una posición; evalúa colección, clave y valor una sola vez.
    SetIndex {
        collection: Expr,
        key: Expr,
        value: Expr,
    },
    Assign {
        name: String,
        annotation: Option<TypeName>,
        expr: Expr,
        binding: BindingKind,
        visibility: Visibility,
    },
    AugAssign {
        name: String,
        op: AugOp,
        expr: Expr,
    },
    Expr(Expr),
    Return(Option<Expr>),
    If {
        branches: Vec<(Expr, Vec<Stmt>)>,
        else_body: Vec<Stmt>,
    },
    While {
        cond: Expr,
        body: Vec<Stmt>,
    },
    ForRange {
        var: String,
        start: Expr,
        stop: Expr,
        body: Vec<Stmt>,
    },
    Break,
    Continue,
    Pass,
}

#[derive(Debug, Clone)]
/// Parámetro de función: name identifica la variable y ty conserva su tipo anotado.
pub struct Param {
    pub name: String,
    pub ty: TypeName,
}

#[derive(Debug, Clone)]
/// Función en el AST: name, parámetros ordenados params, retorno ret y cuerpo body.
pub struct FunctionDef {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: TypeName,
    pub body: Vec<Stmt>,
    /// Controla qué módulos pueden importar la función; no altera llamadas locales.
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Default)]
/// Raíz del AST: functions agrupa definiciones y body las sentencias superiores.
pub struct Program {
    pub functions: Vec<FunctionDef>,
    pub body: Vec<Stmt>,
}
