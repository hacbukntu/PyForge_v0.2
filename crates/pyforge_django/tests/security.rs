//! Regresiones de seguridad sobre SQLite real en memoria y HTTP sin abrir puertos.
use axum::{
    body::{to_bytes, Body},
    http::Request,
};
use pyforge_django::{
    orm::{Model, Orm},
    server::{self, Route},
};
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn sql_parametrizado_y_claves_ocultas() {
    let mut orm = Orm::open("sqlite::memory:").await.unwrap();
    let schema:Model=serde_json::from_value(json!({"nombre":"Usuario","campos":{"correo":{"tipo":"cadena","unique":true},"clave":{"tipo":"clave_segura"}}})).unwrap();
    orm.register(schema).await.unwrap();
    let data =
        json!({"correo":"'; DROP TABLE Usuario; --","clave":"Una clave suficientemente larga"});
    let row = orm
        .create("Usuario", data.as_object().unwrap())
        .await
        .unwrap();
    assert!(row.get("clave").is_none());
    assert!(orm
        .create("Usuario", data.as_object().unwrap())
        .await
        .is_err());
    assert_eq!(
        orm.filter(
            "Usuario",
            json!({"correo":data["correo"]}).as_object().unwrap()
        )
        .await
        .unwrap()
        .len(),
        1
    );
    assert!(orm
        .filter("Usuario", json!({"clave":"x"}).as_object().unwrap())
        .await
        .is_err());
    assert!(orm
        .filter(
            "Usuario; DROP TABLE Usuario",
            json!({}).as_object().unwrap()
        )
        .await
        .is_err());
}

#[tokio::test]
async fn csrf_antes_de_la_vista_y_origen_exacto() {
    let route = Route {
        path: "/".into(),
        name: "inicio".into(),
        view: Arc::new(|p| Ok(json!({"tipo":"json","datos":p}))),
    };
    let app = server::router(vec![route], "http://127.0.0.1:8000".into()).unwrap();
    let response = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let cookie = response.headers()["set-cookie"]
        .to_str()
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .to_owned();
    let body: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 65536).await.unwrap()).unwrap();
    for (origin, token, status) in [
        ("http://127.0.0.1:8000", "", 403),
        (
            "https://ajeno.invalid",
            body["CSRF_TOKEN"].as_str().unwrap(),
            403,
        ),
        (
            "http://127.0.0.1:8000",
            body["CSRF_TOKEN"].as_str().unwrap(),
            200,
        ),
    ] {
        let request = Request::builder()
            .method("POST")
            .uri("/")
            .header("cookie", &cookie)
            .header("origin", origin)
            .header("x-csrf-token", token)
            .header("content-type", "application/json")
            .body(Body::from("{}"))
            .unwrap();
        assert_eq!(
            app.clone()
                .oneshot(request)
                .await
                .unwrap()
                .status()
                .as_u16(),
            status
        );
    }
}

#[test]
fn abi_c_json_y_propiedad_de_memoria() {
    use pyforge_django::*;
    use std::ffi::{CStr, CString};
    unsafe {
        let input=CString::new(r#"{"operacion":"renderizar","plantilla":"{{ nombre }}","contexto":{"nombre":"<script>"}}"#).unwrap();
        let output = pyforge_django_call(input.as_ptr());
        let data: Value = serde_json::from_slice(CStr::from_ptr(output).to_bytes()).unwrap();
        assert_eq!(data["ok"], true);
        assert_eq!(data["respuesta"]["cuerpo"], "&lt;script&gt;");
        pyforge_django_free(output);
        let url = CString::new("sqlite::memory:").unwrap();
        let context = pyforge_django_new(url.as_ptr());
        assert!(!context.is_null());
        let input = CString::new(r#"{"operacion":"todos","modelo":"NoExiste"}"#).unwrap();
        let output = pyforge_django_orm(context, input.as_ptr());
        let data: Value = serde_json::from_slice(CStr::from_ptr(output).to_bytes()).unwrap();
        assert_eq!(data["ok"], false);
        pyforge_django_free(output);
        pyforge_django_destroy(context);
    }
}
