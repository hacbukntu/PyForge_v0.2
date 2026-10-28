//! ORM pequeño basado en metadatos registrados por el anfitrión, sin interfaz de SQL libre.
//! Solo los identificadores validados se interpolan; todos los valores usan bind de sqlx.
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use sqlx::{sqlite::SqlitePoolOptions, QueryBuilder, Row, Sqlite, SqlitePool};
use std::collections::BTreeMap;

/// Tipos persistentes iniciales. Las colecciones y relaciones requieren migraciones posteriores.
#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    Cadena,
    Entero,
    Flotante,
    Booleano,
    ClaveSegura,
}
/// La unicidad forma parte del esquema, no de una comprobación susceptible a carreras.
#[derive(Clone, Serialize, Deserialize)]
pub struct Field {
    pub tipo: FieldType,
    #[serde(default)]
    pub unique: bool,
}
/// Modelo registrado al arrancar; no se acepta una definición enviada por un visitante.
#[derive(Clone, Serialize, Deserialize)]
pub struct Model {
    pub nombre: String,
    pub campos: BTreeMap<String, Field>,
}
/// Conexiones y catálogo privados: ninguna operación pública acepta fragmentos SQL.
pub struct Orm {
    pool: SqlitePool,
    models: BTreeMap<String, Model>,
}

/// Nombres ASCII de tamaño acotado; rechaza puntuación, palabras SQLite reservadas y id explícito.
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 63
        && s.as_bytes()[0].is_ascii_alphabetic()
        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        && !s.to_ascii_lowercase().starts_with("sqlite_")
}
impl Orm {
    /// Abre SQLite con una conexión; también permite sqlite::memory: en pruebas.
    pub async fn open(url: &str) -> Result<Self, String> {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect(url)
            .await
            .map_err(|_| "No se pudo abrir SQLite")?;
        Ok(Self {
            pool,
            models: BTreeMap::new(),
        })
    }
    /// Crea una tabla nueva de forma atómica. No modifica silenciosamente esquemas existentes.
    pub async fn register(&mut self, model: Model) -> Result<(), String> {
        if !identifier(&model.nombre)
            || model.campos.is_empty()
            || model.campos.len() > 32
            || self
                .models
                .keys()
                .any(|name| name.eq_ignore_ascii_case(&model.nombre))
            || model
                .campos
                .keys()
                .any(|k| !identifier(k) || k.eq_ignore_ascii_case("id"))
        {
            return Err("Modelo inválido o duplicado".into());
        }
        let mut names = std::collections::HashSet::new();
        if model
            .campos
            .keys()
            .any(|name| !names.insert(name.to_ascii_lowercase()))
        {
            return Err("Campos duplicados".into());
        }
        let mut sql = format!("CREATE TABLE \"{}\" (id INTEGER PRIMARY KEY", model.nombre);
        for (name, field) in &model.campos {
            let ty = match field.tipo {
                FieldType::Entero | FieldType::Booleano => "INTEGER",
                FieldType::Flotante => "REAL",
                _ => "TEXT",
            };
            sql.push_str(&format!(
                ", \"{name}\" {ty} NOT NULL{}",
                if field.unique { " UNIQUE" } else { "" }
            ));
        }
        sql.push(')');
        // Reabrir permite el esquema idéntico; cualquier cambio exige una migración explícita.
        let existing: Option<String> = sqlx::query_scalar(
            "SELECT sql FROM sqlite_master WHERE type = 'table' AND name = ? COLLATE NOCASE",
        )
        .bind(&model.nombre)
        .fetch_optional(&self.pool)
        .await
        .map_err(|_| "No se pudo leer el esquema")?;
        match existing {
            Some(previous) if previous != sql => {
                return Err("El esquema existente necesita una migración".into())
            }
            Some(_) => {}
            None => {
                sqlx::query(&sql)
                    .execute(&self.pool)
                    .await
                    .map_err(|_| "No se pudo crear el modelo")?;
            }
        }
        self.models.insert(model.nombre.clone(), model);
        Ok(())
    }
    /// Valida tipo y tamaño; clave_segura transforma automáticamente el texto en Argon2id.
    fn value(field: &Field, value: &Value) -> Result<Value, String> {
        Ok(match field.tipo {
            FieldType::Cadena if value.as_str().is_some_and(|s| s.len() <= 8192) => value.clone(),
            FieldType::Entero if value.as_i64().is_some() => value.clone(),
            FieldType::Flotante if value.as_f64().is_some() => value.clone(),
            FieldType::Booleano if value.is_boolean() => value.clone(),
            FieldType::ClaveSegura => Value::String(crate::security::hash_password(
                value.as_str().ok_or("Clave inválida")?,
            )?),
            _ => return Err("Tipo o tamaño inválido".into()),
        })
    }
    /// Inserta todos los campos declarados; devuelve una fila que excluye claves y hashes.
    pub async fn create(&self, model: &str, input: &Map<String, Value>) -> Result<Value, String> {
        let schema = self.models.get(model).ok_or("Modelo desconocido")?;
        if input.len() != schema.campos.len() {
            return Err("Faltan o sobran campos".into());
        }
        let values: Vec<Value> = schema
            .campos
            .iter()
            .map(|(k, f)| Self::value(f, input.get(k).ok_or("Campo ausente")?))
            .collect::<Result<_, _>>()?;
        let mut q = QueryBuilder::<Sqlite>::new(format!("INSERT INTO \"{model}\" ("));
        q.push(
            schema
                .campos
                .keys()
                .map(|k| format!("\"{k}\""))
                .collect::<Vec<_>>()
                .join(","),
        );
        q.push(") VALUES (");
        for (i, v) in values.into_iter().enumerate() {
            if i > 0 {
                q.push(",");
            }
            bind(&mut q, v);
        }
        q.push(")");
        let id = q
            .build()
            .execute(&self.pool)
            .await
            .map_err(|_| "No se pudo guardar: datos o unicidad inválidos")?
            .last_insert_rowid();
        let rows = self
            .filter(model, &Map::from_iter([("id".into(), Value::from(id))]))
            .await?;
        rows.into_iter().next().ok_or("Fila ausente".into())
    }
    /// Igualdad parametrizada, orden estable y límite de 100 filas; nunca filtra contraseñas.
    pub async fn filter(
        &self,
        model: &str,
        filters: &Map<String, Value>,
    ) -> Result<Vec<Value>, String> {
        let schema = self.models.get(model).ok_or("Modelo desconocido")?;
        let mut q = QueryBuilder::<Sqlite>::new(format!("SELECT * FROM \"{model}\" WHERE 1=1"));
        for (key, value) in filters {
            let value = if key == "id" {
                Value::from(value.as_i64().ok_or("id inválido")?)
            } else {
                let field = schema.campos.get(key).ok_or("Campo desconocido")?;
                if matches!(field.tipo, FieldType::ClaveSegura) {
                    return Err("No se permite consultar claves".into());
                }
                Self::value(field, value)?
            };
            q.push(format!(" AND \"{key}\" = "));
            bind(&mut q, value);
        }
        q.push(" ORDER BY id LIMIT 100");
        let rows = q
            .build()
            .fetch_all(&self.pool)
            .await
            .map_err(|_| "No se pudo consultar")?;
        rows.into_iter()
            .map(|row| {
                let mut data = Map::new();
                data.insert(
                    "id".into(),
                    Value::from(
                        row.try_get::<i64, _>("id")
                            .map_err(|_| "Esquema incompatible")?,
                    ),
                );
                for (key, field) in &schema.campos {
                    let value = match field.tipo {
                        FieldType::ClaveSegura => continue,
                        FieldType::Cadena => Value::from(
                            row.try_get::<String, _>(key.as_str())
                                .map_err(|_| "Esquema incompatible")?,
                        ),
                        FieldType::Entero => Value::from(
                            row.try_get::<i64, _>(key.as_str())
                                .map_err(|_| "Esquema incompatible")?,
                        ),
                        FieldType::Booleano => Value::from(
                            row.try_get::<bool, _>(key.as_str())
                                .map_err(|_| "Esquema incompatible")?,
                        ),
                        FieldType::Flotante => Value::from(
                            row.try_get::<f64, _>(key.as_str())
                                .map_err(|_| "Esquema incompatible")?,
                        ),
                    };
                    data.insert(key.clone(), value);
                }
                Ok(Value::Object(data))
            })
            .collect()
    }
}
/// Enlaza valores ya validados sin introducirlos en el texto SQL.
fn bind(q: &mut QueryBuilder<'_, Sqlite>, value: Value) {
    match value {
        Value::String(s) => {
            q.push_bind(s);
        }
        Value::Bool(b) => {
            q.push_bind(b);
        }
        Value::Number(n) if n.is_i64() => {
            q.push_bind(n.as_i64().unwrap());
        }
        Value::Number(n) => {
            q.push_bind(n.as_f64().unwrap());
        }
        _ => unreachable!("valor validado"),
    }
}
