use serde::Serialize;
use sqlx::FromRow;

use crate::db::{Insertable, Model, Value};

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Upload {
    pub id: String,
    pub user_id: Option<String>,
    pub filename: String,
    pub original_name: String,
    pub content_type: Option<String>,
    pub size: i64,
    pub created_at: String,
}

impl Model for Upload {
    const TABLE: &'static str = "uploads";
}

impl Insertable for Upload {
    fn id(&self) -> &str {
        &self.id
    }

    // `created_at` no va aquí: se deja fuera del INSERT a propósito para que
    // el DEFAULT CURRENT_TIMESTAMP de la migración sea quien lo asigne.
    fn fields(&self) -> Vec<(&'static str, Value)> {
        vec![
            ("user_id", Value::OptStr(self.user_id.clone())),
            ("filename", Value::Str(self.filename.clone())),
            ("original_name", Value::Str(self.original_name.clone())),
            ("content_type", Value::OptStr(self.content_type.clone())),
            ("size", Value::Int(self.size)),
        ]
    }
}
