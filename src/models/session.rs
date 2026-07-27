use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub expires_at: i64,
}
