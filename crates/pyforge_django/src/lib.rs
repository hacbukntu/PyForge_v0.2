//! Base Rust de pyforge_django. ABI C versionada y separada del runtime AOT actual.
//! No registra por sí sola una palabra del parser ni convierte funciones ST en callbacks.
pub mod orm;
pub mod security;
pub mod server;
pub mod templates;
use serde_json::{json, Value};
use std::ffi::{c_char, CStr, CString};

/// Nombre del módulo y versión del convenio; el anfitrión puede consultarlos sin memoria propia.
#[no_mangle]
pub extern "C" fn pyforge_django_abi_version() -> u32 {
    1
}

/// Operaciones puras para la futura tabla de importaciones. Devuelve JSON UTF-8 propio.
/// # Safety
/// `input` debe apuntar a una cadena C UTF-8 terminada en NUL, válida durante la llamada.
/// La salida debe liberarse exactamente una vez con pyforge_django_free.
#[no_mangle]
pub unsafe extern "C" fn pyforge_django_call(input: *const c_char) -> *mut c_char {
    let result = std::panic::catch_unwind(|| {
        if input.is_null() {
            return Err("Entrada nula".to_owned());
        }
        let bytes = unsafe { CStr::from_ptr(input) }.to_bytes();
        if bytes.len() > 65536 {
            return Err("Entrada demasiado grande".into());
        }
        let value: Value = serde_json::from_slice(bytes).map_err(|_| "JSON inválido")?;
        match value["operacion"].as_str().unwrap_or("") {
            "responder_json" if value["datos"].is_object() => {
                Ok(json!({"tipo":"json","datos":value["datos"]}))
            }
            "redirigir" => {
                let url = value["url"].as_str().ok_or("URL inválida")?;
                if !url.starts_with('/')
                    || url.starts_with("//")
                    || url.contains(['\\', '\r', '\n'])
                {
                    return Err("La redirección debe ser local".into());
                }
                Ok(json!({"tipo":"redireccion","url":url}))
            }
            "renderizar" => {
                let source = value["plantilla"].as_str().ok_or("Plantilla inválida")?;
                let templates = templates::Templates::new(&[("pagina", source)])?;
                Ok(json!({"tipo":"html","cuerpo":templates.render("pagina",&value["contexto"])?}))
            }
            _ => Err("Operación desconocida en esta ABI".into()),
        }
    });
    let envelope = match result {
        Ok(Ok(v)) => json!({"ok":true,"respuesta":v}),
        Ok(Err(e)) => json!({"ok":false,"error":e}),
        Err(_) => json!({"ok":false,"error":"Error interno"}),
    };
    CString::new(envelope.to_string()).unwrap().into_raw()
}

/// Libera únicamente una salida de pyforge_django_call; NULL es aceptable.
/// # Safety
/// El puntero debe proceder de este crate y no haberse liberado previamente.
#[no_mangle]
pub unsafe extern "C" fn pyforge_django_free(value: *mut c_char) {
    if !value.is_null() {
        drop(unsafe { CString::from_raw(value) });
    }
}

/// Estado opaco propiedad del anfitrión. No comparte representaciones internas Rust por la ABI.
pub struct Django {
    runtime: tokio::runtime::Runtime,
    orm: std::sync::Mutex<orm::Orm>,
    routes: std::sync::Mutex<Vec<server::Route>>,
    execution: std::sync::Arc<std::sync::Mutex<()>>,
}

/// Crea un contexto con SQLite; NULL indica error. No se aceptan URLs de otras bases.
/// # Safety
/// `url` debe ser una cadena C válida. El contexto debe liberarse una vez, tras detener HTTP.
#[no_mangle]
pub unsafe extern "C" fn pyforge_django_new(url: *const c_char) -> *mut Django {
    std::panic::catch_unwind(|| {
        if url.is_null() {
            return std::ptr::null_mut();
        }
        let Ok(url) = unsafe { CStr::from_ptr(url) }.to_str() else {
            return std::ptr::null_mut();
        };
        if !url.starts_with("sqlite:") {
            return std::ptr::null_mut();
        }
        let Ok(runtime) = tokio::runtime::Runtime::new() else {
            return std::ptr::null_mut();
        };
        let Ok(orm) = runtime.block_on(orm::Orm::open(url)) else {
            return std::ptr::null_mut();
        };
        Box::into_raw(Box::new(Django {
            runtime,
            orm: std::sync::Mutex::new(orm),
            routes: std::sync::Mutex::new(Vec::new()),
            execution: std::sync::Arc::new(std::sync::Mutex::new(())),
        }))
    })
    .unwrap_or(std::ptr::null_mut())
}

/// Destruye el contexto. No debe coincidir con otra llamada ni ejecutarse dentro de Tokio.
/// # Safety
/// `context` debe ser NULL o un puntero vivo devuelto por pyforge_django_new.
#[no_mangle]
pub unsafe extern "C" fn pyforge_django_destroy(context: *mut Django) {
    if !context.is_null() {
        drop(unsafe { Box::from_raw(context) });
    }
}

/// Ejecuta registrar/crear/filtrar/obtener/todos sin exponer texto SQL.
/// # Safety
/// El contexto y la cadena C deben estar vivos; liberar la salida con pyforge_django_free.
#[no_mangle]
pub unsafe extern "C" fn pyforge_django_orm(
    context: *mut Django,
    input: *const c_char,
) -> *mut c_char {
    let result = std::panic::catch_unwind(|| -> Result<Value, String> {
        let context = unsafe { context.as_ref() }.ok_or("Contexto nulo")?;
        if input.is_null() {
            return Err("Entrada nula".into());
        }
        let bytes = unsafe { CStr::from_ptr(input) }.to_bytes();
        if bytes.len() > 65536 {
            return Err("Entrada demasiado grande".into());
        }
        let data: Value = serde_json::from_slice(bytes).map_err(|_| "JSON inválido")?;
        let mut orm = context.orm.lock().map_err(|_| "Contexto ocupado")?;
        context.runtime.block_on(async {
            let model = data["modelo"].as_str().unwrap_or("");
            match data["operacion"].as_str().unwrap_or("") {
                "registrar" => {
                    orm.register(
                        serde_json::from_value(data["esquema"].clone())
                            .map_err(|_| "Esquema inválido")?,
                    )
                    .await?;
                    Ok(Value::Null)
                }
                "crear" => {
                    orm.create(model, data["datos"].as_object().ok_or("Datos inválidos")?)
                        .await
                }
                "filtrar" => Ok(Value::Array(
                    orm.filter(
                        model,
                        data["filtros"].as_object().ok_or("Filtros inválidos")?,
                    )
                    .await?,
                )),
                "todos" => Ok(Value::Array(
                    orm.filter(model, &serde_json::Map::new()).await?,
                )),
                "obtener" => {
                    let filters = serde_json::Map::from_iter([("id".into(), data["id"].clone())]);
                    Ok(orm
                        .filter(model, &filters)
                        .await?
                        .into_iter()
                        .next()
                        .unwrap_or(Value::Null))
                }
                _ => Err("Operación ORM desconocida".into()),
            }
        })
    });
    let envelope = match result {
        Ok(Ok(v)) => json!({"ok":true,"respuesta":v}),
        Ok(Err(e)) => json!({"ok":false,"error":e}),
        Err(_) => json!({"ok":false,"error":"Error interno"}),
    };
    CString::new(envelope.to_string()).unwrap().into_raw()
}

/// Callback C: devuelve una cadena JSON que permanece viva hasta llamar a su liberador.
pub type Callback = unsafe extern "C" fn(*const c_char) -> *mut c_char;
pub type Release = unsafe extern "C" fn(*mut c_char);

/// Registra una vista y serializa sus invocaciones: el anfitrión no tiene que ser reentrante.
/// # Safety
/// Cadenas C válidas; callbacks vivos hasta detener el servidor y destruir el contexto.
/// La vista no debe volver a registrar rutas ni llamar a escuchar. Su liberador debe aceptar su salida.
#[no_mangle]
pub unsafe extern "C" fn pyforge_django_route(
    context: *mut Django,
    path: *const c_char,
    name: *const c_char,
    callback: Option<Callback>,
    release: Option<Release>,
) -> i32 {
    std::panic::catch_unwind(|| -> Result<(), ()> {
        let ctx = unsafe { context.as_ref() }.ok_or(())?;
        if path.is_null() || name.is_null() {
            return Err(());
        }
        let path = unsafe { CStr::from_ptr(path) }
            .to_str()
            .map_err(|_| ())?
            .to_owned();
        let name = unsafe { CStr::from_ptr(name) }
            .to_str()
            .map_err(|_| ())?
            .to_owned();
        let callback = callback.ok_or(())?;
        let release = release.ok_or(())?;
        let lock = ctx.execution.clone();
        let view = std::sync::Arc::new(move |input: Value| -> Result<Value, String> {
            let _guard = lock.lock().map_err(|_| "Vista indisponible")?;
            let input = CString::new(input.to_string()).map_err(|_| "Petición inválida")?;
            let output = unsafe { callback(input.as_ptr()) };
            if output.is_null() {
                return Err("Vista sin respuesta".into());
            }
            let bytes = unsafe { CStr::from_ptr(output) }.to_bytes();
            let result = if bytes.len() > 1024 * 1024 {
                Err("Respuesta demasiado grande".into())
            } else {
                serde_json::from_slice(bytes).map_err(|_| "Respuesta inválida".into())
            };
            unsafe { release(output) };
            result
        });
        let mut routes = ctx.routes.lock().map_err(|_| ())?;
        let mut next = routes.clone();
        next.push(server::Route { path, name, view });
        let _ = server::router(next.clone(), "http://127.0.0.1:8000".into()).map_err(|_| ())?;
        *routes = next;
        Ok(())
    })
    .map_or(-1, |r| if r.is_ok() { 0 } else { -1 })
}

/// Arranca HTTP hasta Ctrl+C. `origin` debe coincidir con el origen público del navegador.
/// # Safety
/// El contexto y la cadena C deben permanecer vivos durante toda la ejecución.
#[no_mangle]
pub unsafe extern "C" fn pyforge_django_listen(
    context: *mut Django,
    port: u16,
    origin: *const c_char,
) -> i32 {
    std::panic::catch_unwind(|| -> Result<(), String> {
        let ctx = unsafe { context.as_ref() }.ok_or("Contexto nulo")?;
        if origin.is_null() {
            return Err("Origen nulo".into());
        }
        let origin = unsafe { CStr::from_ptr(origin) }
            .to_str()
            .map_err(|_| "Origen inválido")?
            .to_owned();
        let routes = ctx
            .routes
            .lock()
            .map_err(|_| "Rutas indisponibles")?
            .clone();
        ctx.runtime.block_on(server::listen(routes, port, origin))
    })
    .map_or(-1, |r| if r.is_ok() { 0 } else { -1 })
}
