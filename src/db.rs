use std::borrow::Cow;

use sqlx::any::{AnyArguments, AnyRow};
use sqlx::query::Query;
use sqlx::{AnyPool, FromRow};
use uuid::Uuid;

/// Estado compartido de la aplicación: el pool de conexiones agnóstico más
/// la información necesaria para adaptar las consultas al motor conectado.
#[derive(Clone)]
pub struct AppState {
    pub pool: AnyPool,
    pub is_postgres: bool,
}

impl AppState {
    pub fn new(pool: AnyPool, is_postgres: bool) -> Self {
        Self { pool, is_postgres }
    }

    /// Reescribe los placeholders `?` de una consulta a la sintaxis nativa
    /// del backend conectado.
    ///
    /// `sqlx::Any` (a diferencia de lo que sugiere la doc original de este
    /// proyecto) no traduce placeholders por su cuenta: SQLite los entiende
    /// como `?` de forma nativa, pero Postgres exige `$1, $2, ...` y responde
    /// con un error de sintaxis ante un `?` literal. Esta función es la que
    /// de verdad permite escribir `?` en todos los controladores y que
    /// funcione contra ambos motores con solo cambiar `DATABASE_URL`.
    pub fn sql<'a>(&self, query: &'a str) -> Cow<'a, str> {
        if !self.is_postgres || !query.contains('?') {
            return Cow::Borrowed(query);
        }

        let mut rewritten = String::with_capacity(query.len() + 8);
        let mut placeholder_count = 0u32;
        for ch in query.chars() {
            if ch == '?' {
                placeholder_count += 1;
                rewritten.push('$');
                rewritten.push_str(&placeholder_count.to_string());
            } else {
                rewritten.push(ch);
            }
        }
        Cow::Owned(rewritten)
    }
}

/// Genera un identificador nuevo (UUIDv7) para una fila que la aplicación
/// inserta ella misma, en vez de depender de un autoincremental de la base
/// de datos (cuya sintaxis difiere entre SQLite y Postgres). UUIDv7 es
/// ordenable por tiempo de creación, así que los IDs se mantienen
/// aproximadamente secuenciales.
pub fn new_id() -> String {
    Uuid::now_v7().to_string()
}

/// Helpers CRUD genéricos para eliminar el `SELECT ... WHERE id = ?` /
/// `DELETE ... WHERE id = ?` repetido en cada controlador. No es un ORM real
/// (sin relaciones ni query builder): `find`/`all`/`delete` funcionan para
/// cualquier modelo con solo declarar `TABLE`, porque `#[derive(FromRow)]`
/// mapea columnas por nombre, no por posición — un `SELECT *` genérico
/// siempre encuentra las columnas que el struct necesita.
// async fn en un trait no puede declarar `Send` en el Future que genera, lo
// cual sí importa para traits usados vía `dyn Trait`. No es nuestro caso:
// `Model`/`Insertable` siempre se usan por dispatch estático (`T: Model`),
// nunca como `dyn Model`, así que el Future queda Send igual (se infiere del
// cuerpo concreto de cada impl) y el lint no aplica aquí.
#[allow(async_fn_in_trait)]
pub trait Model: Sized + for<'r> FromRow<'r, AnyRow> + Send + Unpin {
    const TABLE: &'static str;

    async fn find(state: &AppState, id: &str) -> Result<Option<Self>, sqlx::Error> {
        let sql = format!("SELECT * FROM {} WHERE id = ?", Self::TABLE);
        sqlx::query_as::<_, Self>(&state.sql(&sql))
            .bind(id)
            .fetch_optional(&state.pool)
            .await
    }

    async fn all(state: &AppState) -> Result<Vec<Self>, sqlx::Error> {
        let sql = format!("SELECT * FROM {} ORDER BY id", Self::TABLE);
        sqlx::query_as::<_, Self>(&sql).fetch_all(&state.pool).await
    }

    async fn delete(state: &AppState, id: &str) -> Result<(), sqlx::Error> {
        let sql = format!("DELETE FROM {} WHERE id = ?", Self::TABLE);
        sqlx::query(&state.sql(&sql)).bind(id).execute(&state.pool).await?;
        Ok(())
    }
}

/// Los pocos tipos de columna que los modelos de este proyecto realmente
/// usan (no es una reescritura genérica de `sqlx::Encode`: Rust no tiene
/// reflexión, así que `Insertable::fields()` sigue siendo código a mano por
/// modelo — lo que este enum evita es escribir el `INSERT`/`UPDATE` y los
/// `.bind()` de cada columna a mano en cada controlador).
pub enum Value {
    Str(String),
    OptStr(Option<String>),
    Int(i64),
}

fn bind_value<'q>(
    query: Query<'q, sqlx::Any, AnyArguments<'q>>,
    value: Value,
) -> Query<'q, sqlx::Any, AnyArguments<'q>> {
    match value {
        Value::Str(s) => query.bind(s),
        Value::OptStr(s) => query.bind(s),
        Value::Int(n) => query.bind(n),
    }
}

/// Extiende `Model` con `create`/`update` para modelos que exponen sus
/// columnas insertables vía `fields()`. Deliberadamente separado de `Model`:
/// no todos los modelos necesitan un `create`/`update` genérico (p. ej.
/// `User` tiene lógica propia para el hash de contraseña y el mensaje de
/// "correo duplicado", así que solo implementa `Model`).
#[allow(async_fn_in_trait)]
pub trait Insertable: Model {
    fn id(&self) -> &str;

    /// Pares (columna, valor) a insertar/actualizar, sin incluir `id`.
    fn fields(&self) -> Vec<(&'static str, Value)>;

    async fn create(&self, state: &AppState) -> Result<(), sqlx::Error> {
        let fields = self.fields();
        let mut columns = vec!["id"];
        columns.extend(fields.iter().map(|(column, _)| *column));
        let placeholders = vec!["?"; columns.len()].join(", ");
        let sql = format!(
            "INSERT INTO {} ({}) VALUES ({})",
            Self::TABLE,
            columns.join(", "),
            placeholders
        );

        let rewritten = state.sql(&sql);
        let mut query = sqlx::query(&rewritten).bind(self.id().to_string());
        for (_, value) in fields {
            query = bind_value(query, value);
        }
        query.execute(&state.pool).await?;
        Ok(())
    }

    async fn update(&self, state: &AppState) -> Result<(), sqlx::Error> {
        let fields = self.fields();
        let set_clause = fields
            .iter()
            .map(|(column, _)| format!("{} = ?", column))
            .collect::<Vec<_>>()
            .join(", ");
        let sql = format!("UPDATE {} SET {} WHERE id = ?", Self::TABLE, set_clause);

        let rewritten = state.sql(&sql);
        let mut query = sqlx::query(&rewritten);
        for (_, value) in fields {
            query = bind_value(query, value);
        }
        query = query.bind(self.id().to_string());
        query.execute(&state.pool).await?;
        Ok(())
    }
}
