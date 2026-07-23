use serde::Serialize;
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
}

impl User {
    pub fn hash_password(password: &str) -> Result<String, bcrypt::BcryptError> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
    }

    pub fn verify_password(&self, password: &str) -> bool {
        bcrypt::verify(password, &self.password_hash).unwrap_or(false)
    }
}
