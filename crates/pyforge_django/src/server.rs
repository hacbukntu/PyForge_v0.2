//! HTTP con rutas explícitas, límite de cuerpo y CSRF ligado a sesiones del servidor.
use crate::security;
use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::{HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::any,
    Router,
};
use serde_json::{json, Value};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

/// Vista del anfitrión: recibe una petición JSON, devuelve una respuesta JSON.
/// El adaptador del compilador deberá convertir diccionarios ST a este convenio.
pub type View = Arc<dyn Fn(Value) -> Result<Value, String> + Send + Sync>;
/// Las rutas se configuran antes de escuchar; no existe ejecución de código recibido por HTTP.
#[derive(Clone)]
pub struct Route {
    pub path: String,
    pub name: String,
    pub view: View,
}
#[derive(Clone)]
struct App {
    routes: Arc<Vec<Route>>,
    sessions: Arc<Mutex<HashMap<String, (String, Instant)>>>,
    origin: String,
    capacity: Arc<tokio::sync::Semaphore>,
}

/// Valida nombres y rutas antes de construir el router. Solo rutas literales en esta base.
pub fn router(routes: Vec<Route>, origin: String) -> Result<Router, String> {
    if !(origin.starts_with("https://") || origin.starts_with("http://127.0.0.1:"))
        || origin.ends_with('/')
    {
        return Err("Origen inválido".into());
    }
    let mut paths = std::collections::HashSet::new();
    let mut names = std::collections::HashSet::new();
    for route in &routes {
        if !route.path.starts_with('/')
            || route.path.starts_with("//")
            || route.path.contains(['{', '}', '?', '#', '\\'])
            || !paths.insert(route.path.clone())
            || !names.insert(route.name.clone())
        {
            return Err("Ruta o nombre inválido/duplicado".into());
        }
    }
    let state = App {
        routes: Arc::new(routes),
        sessions: Arc::new(Mutex::new(HashMap::new())),
        origin,
        capacity: Arc::new(tokio::sync::Semaphore::new(32)),
    };
    // Un despachador propio evita que una cadena externa active patrones internos de axum.
    Ok(Router::new().fallback(any(dispatch)).with_state(state))
}

/// Escucha únicamente loopback; producción necesita un proxy HTTPS de confianza.
pub async fn listen(routes: Vec<Route>, port: u16, origin: String) -> Result<(), String> {
    if port == 0 {
        return Err("Puerto inválido".into());
    }
    let app = router(routes, origin)?;
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, port))
        .await
        .map_err(|_| "No se pudo abrir el puerto")?;
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await
        .map_err(|_| "Servidor interrumpido".into())
}

/// Comprueba la sesión y el token antes de invocar cualquier vista POST.
async fn dispatch(State(app): State<App>, request: Request) -> Response {
    let Ok(permit) = app.capacity.clone().try_acquire_owned() else {
        return StatusCode::TOO_MANY_REQUESTS.into_response();
    };
    let path = request.uri().path().to_owned();
    let Some(route) = app.routes.iter().find(|r| r.path == path) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let method = request.method().as_str().to_owned();
    if method != "GET" && method != "POST" {
        return StatusCode::METHOD_NOT_ALLOWED.into_response();
    }
    let session = request
        .headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(';')
        .find_map(|v| v.trim().strip_prefix("st_session="))
        .unwrap_or("")
        .to_owned();
    let supplied = request
        .headers()
        .get("x-csrf-token")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();
    let origin = request
        .headers()
        .get("origin")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_owned();
    let (session, csrf) = {
        let Ok(mut sessions) = app.sessions.lock() else {
            return StatusCode::SERVICE_UNAVAILABLE.into_response();
        };
        sessions.retain(|_, (_, time)| time.elapsed() < Duration::from_secs(1800));
        if method == "POST" {
            let Some((csrf, _)) = sessions.get(&session) else {
                return StatusCode::FORBIDDEN.into_response();
            };
            if origin != app.origin || !security::valid_csrf(csrf, &supplied) {
                return StatusCode::FORBIDDEN.into_response();
            }
            (session, csrf.clone())
        } else if let Some((csrf, _)) = sessions.get(&session) {
            (session, csrf.clone())
        } else {
            if sessions.len() >= 4096 {
                return StatusCode::TOO_MANY_REQUESTS.into_response();
            }
            let id = security::token();
            let csrf = security::token();
            sessions.insert(id.clone(), (csrf.clone(), Instant::now()));
            (id, csrf)
        }
    };
    // El contrato inicial acepta JSON en POST. Los formularios multipart no están habilitados.
    let query = request.uri().query().unwrap_or("").to_owned();
    if method == "POST"
        && request
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .split(';')
            .next()
            != Some("application/json")
    {
        return StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response();
    }
    let bytes = match to_bytes(request.into_body(), 64 * 1024).await {
        Ok(v) => v,
        Err(_) => return StatusCode::PAYLOAD_TOO_LARGE.into_response(),
    };
    let post = if method == "POST" {
        match serde_json::from_slice::<Value>(&bytes) {
            Ok(v) if v.is_object() => v,
            _ => return StatusCode::BAD_REQUEST.into_response(),
        }
    } else {
        json!({})
    };
    let get: HashMap<String, String> = match serde_urlencoded::from_str(&query) {
        Ok(v) => v,
        Err(_) => return StatusCode::BAD_REQUEST.into_response(),
    };
    let input = json!({"metodo":method,"GET":get,"POST":post,"CSRF_TOKEN":csrf});
    let view = route.view.clone();
    let output = tokio::task::spawn_blocking(move || {
        let _permit = permit;
        view(input)
    })
    .await;
    let mut response = match output {
        Ok(Ok(value)) => response(value),
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let cookie = format!(
        "st_session={session}; Path=/; HttpOnly; SameSite=Strict; Max-Age=1800{}",
        if app.origin.starts_with("https://") {
            "; Secure"
        } else {
            ""
        }
    );
    response
        .headers_mut()
        .insert("set-cookie", HeaderValue::from_str(&cookie).unwrap());
    for (key, value) in [
        ("x-content-type-options", "nosniff"),
        ("x-frame-options", "DENY"),
        ("cache-control", "no-store"),
        ("referrer-policy", "same-origin"),
        (
            "content-security-policy",
            "default-src 'self'; object-src 'none'; frame-ancestors 'none'; base-uri 'self'",
        ),
    ] {
        response
            .headers_mut()
            .insert(key, HeaderValue::from_static(value));
    }
    response
}

/// Traduce únicamente tipos de respuesta conocidos, sin encabezados arbitrarios.
fn response(value: Value) -> Response {
    match value.get("tipo").and_then(Value::as_str) {
        Some("html") => {
            axum::response::Html(value["cuerpo"].as_str().unwrap_or("").to_owned()).into_response()
        }
        Some("json") => axum::Json(value["datos"].clone()).into_response(),
        Some("redireccion") => {
            let url = value["url"].as_str().unwrap_or("");
            if !url.starts_with('/') || url.starts_with("//") || url.contains(['\\', '\r', '\n']) {
                return StatusCode::BAD_REQUEST.into_response();
            }
            match HeaderValue::from_str(url) {
                Ok(header) => {
                    let mut r = Response::new(Body::empty());
                    *r.status_mut() = StatusCode::SEE_OTHER;
                    r.headers_mut().insert("location", header);
                    r
                }
                Err(_) => StatusCode::BAD_REQUEST.into_response(),
            }
        }
        _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
