use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Session {
    pub id: String,
    pub user_id: i64,
    pub expires_at: i64,
}
