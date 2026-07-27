use serde::Serialize;
use sqlx::FromRow;

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
