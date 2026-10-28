//! Plantillas del desarrollador, separadas de los datos recibidos por HTTP.
use serde_json::Value;
use tera::{Context, Tera};

/// El registro es privado: todas las extensiones usan autoescape, incluso nombres sin .html.
#[derive(Default)]
pub struct Templates {
    engine: Tera,
}
impl Templates {
    /// Registra fuentes confiables al iniciar; los visitantes nunca pueden subir plantillas.
    pub fn new(sources: &[(&str, &str)]) -> Result<Self, String> {
        let mut engine = Tera::default();
        engine.autoescape_on(vec![""]);
        engine
            .add_raw_templates(sources.iter().copied())
            .map_err(|_| "Plantilla inválida")?;
        Ok(Self { engine })
    }
    /// Inserta datos como contexto, nunca como código de plantilla ni HTML marcado seguro.
    pub fn render(&self, name: &str, data: &Value) -> Result<String, String> {
        if !data.is_object() {
            return Err("El contexto debe ser un diccionario".into());
        }
        let context = Context::from_serialize(data).map_err(|_| "Contexto inválido")?;
        self.engine
            .render(name, &context)
            .map_err(|_| "No se pudo renderizar la plantilla".into())
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn escapa_todas_las_extensiones() {
        let t = super::Templates::new(&[("pagina", "{{ valor }}")]).unwrap();
        let html = t
            .render(
                "pagina",
                &serde_json::json!({"valor":"<script>alert(1)</script>"}),
            )
            .unwrap();
        assert!(!html.contains("<script>"));
        assert!(html.contains("&lt;script&gt;"));
    }
}
