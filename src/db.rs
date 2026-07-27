use std::borrow::Cow;

use sqlx::AnyPool;
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
