//! ABI 1 experimental para Linux x86-64, monohilo y sin libc/Python.
//! Los identificadores son crecientes y nunca se reutilizan; no son punteros públicos.
//! Hay 32 ranuras compartidas por archivos y buffers de hasta 64 KiB cada uno.
//! El almacenamiento reservado (aprox. 2 MiB BSS) es fijo y separado del heap ST.
#![no_std]
use core::{arch::asm, panic::PanicInfo, ptr};
const COUNT: usize = 32;
const CAPACITY: usize = 65536;
const INVALID: i64 = -10001;
const LIMIT: i64 = -10002;
const RANGE: i64 = -10003;
const UTF8: i64 = -10004;

/// Una ranura libre tiene id cero. kind=1 significa buffer y kind=2 archivo.
#[derive(Clone, Copy)]
struct Slot {
    id: i64,
    kind: u8,
    fd: i64,
    len: usize,
    data: [u8; CAPACITY],
}
static mut SLOTS: [Slot; COUNT] = [Slot {
    id: 0,
    kind: 0,
    fd: -1,
    len: 0,
    data: [0; CAPACITY],
}; COUNT];
static mut NEXT: i64 = 1;
static EMPTY: [u64; 1] = [0];
extern "C" {
    /// Puente al asignador ST: devuelve memoria o termina con su diagnóstico habitual.
    fn pf_native_alloc(size: usize) -> *mut u8;
}

/// Única frontera con Linux. Los punteros son validados por quien prepara la llamada.
unsafe fn syscall(number: i64, a: i64, b: i64, c: i64) -> i64 {
    let mut result = number;
    asm!("syscall", inlateout("rax") result, in("rdi") a, in("rsi") b,
        in("rdx") c, lateout("rcx") _, lateout("r11") _, options(nostack));
    result
}

/// Localiza un recurso vivo del tipo exigido. Solo admite ejecución monohilo.
unsafe fn find(id: i64, kind: u8) -> Option<*mut Slot> {
    let base = ptr::addr_of_mut!(SLOTS).cast::<Slot>();
    for i in 0..COUNT {
        let slot = base.add(i);
        if (*slot).id == id && id > 0 && (kind == 0 || (*slot).kind == kind) {
            return Some(slot);
        }
    }
    None
}

/// Reserva una ranura y un identificador nuevo; el agotamiento no invalida recursos vivos.
unsafe fn reserve(kind: u8) -> Option<*mut Slot> {
    if NEXT == i64::MAX {
        return None;
    }
    let base = ptr::addr_of_mut!(SLOTS).cast::<Slot>();
    for i in 0..COUNT {
        let slot = base.add(i);
        if (*slot).id == 0 {
            (*slot).id = NEXT;
            NEXT += 1;
            (*slot).kind = kind;
            (*slot).fd = -1;
            (*slot).len = 0;
            return Some(slot);
        }
    }
    None
}

/// Borra bytes de buffers y retira la ranura. No promete borrado de todas las copias.
unsafe fn release(slot: *mut Slot) {
    for i in 0..(*slot).len {
        ptr::write_volatile(ptr::addr_of_mut!((*slot).data).cast::<u8>().add(i), 0);
    }
    (*slot).len = 0;
    (*slot).id = 0;
    (*slot).kind = 0;
    (*slot).fd = -1;
}

/// Lee una cadena ST emitida por el compilador: longitud u64 y bytes contiguos.
/// Seguridad: el compilador garantiza puntero válido y vida útil durante la llamada.
unsafe fn text<'a>(value: *const u64) -> &'a [u8] {
    core::slice::from_raw_parts(value.add(1).cast(), *value as usize)
}

/// Versión entera de la ABI para verificar el enlace desde ST.
#[no_mangle]
pub extern "C" fn pf_native_version() -> i64 {
    1
}

/// Abre lectura (0), creación exclusiva (1) o truncado explícito (2).
/// Las rutas son del usuario del proceso; no aplica permisos del auxiliar Python.
/// Seguridad: path cumple la representación de cadena ST; no se conserva su puntero.
#[no_mangle]
pub unsafe extern "C" fn pf_native_open(path: *const u64, mode: i64) -> i64 {
    let source = text(path);
    if source.is_empty() || source.len() > 4095 || source.contains(&0) {
        return RANGE;
    }
    let flags = match mode {
        0 => 0,
        1 => 1 | 64 | 128,
        2 => 1 | 64 | 512,
        _ => return RANGE,
    };
    let slot = match reserve(2) {
        Some(s) => s,
        None => return LIMIT,
    };
    let mut name = [0u8; 4096];
    name[..source.len()].copy_from_slice(source);
    // CLOEXEC y NONBLOCK evitan herencia y bloqueo al abrir recursos especiales.
    let fd = loop {
        let result = syscall(2, name.as_ptr() as i64, flags | 524288 | 2048, 384);
        if result != -4 {
            break result;
        }
    };
    if fd < 0 {
        release(slot);
        return fd;
    }
    (*slot).fd = fd;
    (*slot).id
}

/// Cierra un archivo o libera un buffer. Un alias cerrado falla, aunque se abra otro recurso.
/// Seguridad: solo el runtime monohilo modifica las ranuras.
#[no_mangle]
pub unsafe extern "C" fn pf_native_close(id: i64) -> i64 {
    let slot = match find(id, 0) {
        Some(s) => s,
        None => return INVALID,
    };
    // En Linux close no se reintenta tras EINTR: el descriptor podría haberse liberado.
    let result = if (*slot).kind == 2 {
        syscall(3, (*slot).fd, 0, 0)
    } else {
        0
    };
    release(slot);
    result
}

/// Crea un buffer de tamaño fijo, inicialmente cero; máximo 64 KiB.
#[no_mangle]
pub unsafe extern "C" fn pf_native_buffer(size: i64) -> i64 {
    if size < 0 || size > CAPACITY as i64 {
        return RANGE;
    }
    let slot = match reserve(1) {
        Some(s) => s,
        None => return LIMIT,
    };
    (*slot).len = size as usize;
    for i in 0..size as usize {
        (*slot).data[i] = 0;
    }
    (*slot).id
}

/// Copia los bytes UTF-8 sin conservar referencias a la cadena del llamador.
/// Seguridad: value cumple el contrato de cadena ST.
#[no_mangle]
pub unsafe extern "C" fn pf_native_from_text(value: *const u64) -> i64 {
    let source = text(value);
    if source.len() > CAPACITY {
        return RANGE;
    }
    let id = pf_native_buffer(source.len() as i64);
    if id < 0 {
        return id;
    }
    let slot = find(id, 1).unwrap();
    (&mut (*slot).data)[..source.len()].copy_from_slice(source);
    id
}

/// Lee hasta count bytes; una lectura corta es válida y EOF devuelve un buffer vacío.
#[no_mangle]
pub unsafe extern "C" fn pf_native_read(id: i64, count: i64) -> i64 {
    let file = match find(id, 2) {
        Some(s) => s,
        None => return INVALID,
    };
    if count <= 0 || count > CAPACITY as i64 {
        return RANGE;
    }
    let buffer = match reserve(1) {
        Some(s) => s,
        None => return LIMIT,
    };
    let result = loop {
        let result = syscall(
            0,
            (*file).fd,
            ptr::addr_of_mut!((*buffer).data) as i64,
            count,
        );
        if result != -4 {
            break result;
        }
    };
    if result < 0 {
        release(buffer);
        return result;
    }
    (*buffer).len = result as usize;
    (*buffer).id
}

/// Escribe todo el buffer o devuelve error; un fallo puede dejar escritura parcial.
#[no_mangle]
pub unsafe extern "C" fn pf_native_write(id: i64, bytes: i64) -> i64 {
    let file = match find(id, 2) {
        Some(s) => s,
        None => return INVALID,
    };
    let buffer = match find(bytes, 1) {
        Some(s) => s,
        None => return INVALID,
    };
    let mut offset = 0;
    while offset < (*buffer).len {
        let result = syscall(
            1,
            (*file).fd,
            ptr::addr_of!((*buffer).data).cast::<u8>().add(offset) as i64,
            ((*buffer).len - offset) as i64,
        );
        if result == -4 {
            continue;
        }
        if result < 0 {
            return result;
        }
        if result == 0 {
            return -5;
        }
        offset += result as usize;
    }
    offset as i64
}

/// Devuelve longitud en bytes de un buffer vivo, nunca número de caracteres Unicode.
#[no_mangle]
pub unsafe extern "C" fn pf_native_len(id: i64) -> i64 {
    match find(id, 1) {
        Some(s) => (*s).len as i64,
        None => INVALID,
    }
}

/// Consulta un byte por índice no negativo; no se admiten accesos fuera del buffer.
#[no_mangle]
pub unsafe extern "C" fn pf_native_get(id: i64, index: i64) -> i64 {
    let slot = match find(id, 1) {
        Some(s) => s,
        None => return INVALID,
    };
    if index < 0 || index as usize >= (*slot).len {
        return RANGE;
    }
    (*slot).data[index as usize] as i64
}

/// Modifica un byte (0..255), conservando longitud y propiedad del buffer.
#[no_mangle]
pub unsafe extern "C" fn pf_native_set(id: i64, index: i64, value: i64) -> i64 {
    let slot = match find(id, 1) {
        Some(s) => s,
        None => return INVALID,
    };
    if index < 0 || index as usize >= (*slot).len || !(0..=255).contains(&value) {
        return RANGE;
    }
    (*slot).data[index as usize] = value as u8;
    0
}

/// Valida UTF-8 antes de que el adaptador ST solicite una cadena.
#[no_mangle]
pub unsafe extern "C" fn pf_native_utf8(id: i64) -> i64 {
    let slot = match find(id, 1) {
        Some(s) => s,
        None => return INVALID,
    };
    if core::str::from_utf8(&(&(*slot).data)[..(*slot).len]).is_ok() {
        0
    } else {
        UTF8
    }
}

/// Copia un buffer validado al heap ST. Esa copia conserva la vida útil histórica de cadenas.
/// Un llamador ABI inválido recibe cadena vacía, nunca un entero reinterpretado como puntero.
#[no_mangle]
pub unsafe extern "C" fn pf_native_text(id: i64) -> *const u64 {
    if pf_native_utf8(id) < 0 {
        return EMPTY.as_ptr();
    }
    let slot = find(id, 1).unwrap();
    let output = pf_native_alloc((*slot).len + 8);
    output.cast::<u64>().write((*slot).len as u64);
    ptr::copy_nonoverlapping(
        ptr::addr_of!((*slot).data).cast::<u8>(),
        output.add(8),
        (*slot).len,
    );
    output.cast()
}

/// Un fallo interno termina el proceso sin desenrollar una pila ST a través de Rust.
#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    unsafe {
        syscall(60, 70, 0, 0);
    }
    loop {
        core::hint::spin_loop();
    }
}

// LLVM puede emitir estas operaciones incluso sin std; no se enlaza libc.
/// Copia regiones disjuntas; volatile evita que LLVM regenere una llamada recursiva.
#[no_mangle]
pub unsafe extern "C" fn memcpy(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    for i in 0..n {
        dst.add(i).write_volatile(src.add(i).read_volatile());
    }
    dst
}
/// Llena una región válida con un byte.
#[no_mangle]
pub unsafe extern "C" fn memset(dst: *mut u8, value: i32, n: usize) -> *mut u8 {
    for i in 0..n {
        dst.add(i).write_volatile(value as u8);
    }
    dst
}
/// Copia regiones que pueden solaparse respetando la dirección de avance.
#[no_mangle]
pub unsafe extern "C" fn memmove(dst: *mut u8, src: *const u8, n: usize) -> *mut u8 {
    if (dst as usize) <= src as usize {
        memcpy(dst, src, n);
    } else {
        for i in (0..n).rev() {
            dst.add(i).write_volatile(src.add(i).read_volatile());
        }
    }
    dst
}
/// Compara memoria válida según el convenio C usado por rutinas de core.
#[no_mangle]
pub unsafe extern "C" fn memcmp(a: *const u8, b: *const u8, n: usize) -> i32 {
    for i in 0..n {
        let d = a.add(i).read_volatile() as i32 - b.add(i).read_volatile() as i32;
        if d != 0 {
            return d;
        }
    }
    0
}
