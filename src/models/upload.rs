use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct Upload {
    pub id: i64,
    pub user_id: Option<i64>,
    pub filename: String,
    pub original_name: String,
    pub content_type: Option<String>,
    pub size: i64,
    pub created_at: String,
}
