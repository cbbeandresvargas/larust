use serde::Serialize;
use sqlx::FromRow;

use crate::db::Model;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct User {
    pub id: String,
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

// Solo `Model` (find/all/delete): create/update se quedan escritos a mano en
// user_controller.rs y auth_controller.rs porque tienen lógica propia (hash
// de contraseña, mensaje de "correo duplicado") que Insertable no cubre.
impl Model for User {
    const TABLE: &'static str = "users";
}
