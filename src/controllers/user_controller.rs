use askama::Template;
use axum::{
    extract::{Path, State},
    response::Redirect,
    Form,
};
use serde::Deserialize;
use sqlx::AnyPool;

use crate::error::AppError;
use crate::models::User;

#[derive(Template)]
#[template(path = "users/index.html")]
pub struct UsersTemplate {
    pub users: Vec<User>,
}

#[derive(Template)]
#[template(path = "users/form.html")]
pub struct UserFormTemplate {
    pub user: Option<User>,
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct UserForm {
    pub name: String,
    pub email: String,
}

fn blank_user(id: i64, form: &UserForm) -> User {
    User {
        id,
        name: form.name.clone(),
        email: form.email.clone(),
        password_hash: String::new(),
    }
}

pub async fn index(State(pool): State<AnyPool>) -> Result<UsersTemplate, AppError> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, name, email, password_hash FROM users ORDER BY id",
    )
    .fetch_all(&pool)
    .await?;

    Ok(UsersTemplate { users })
}

pub async fn new() -> UserFormTemplate {
    UserFormTemplate {
        user: None,
        error: None,
    }
}

pub async fn create(
    State(pool): State<AnyPool>,
    Form(form): Form<UserForm>,
) -> Result<Redirect, UserFormTemplate> {
    // Los usuarios creados desde este formulario de administración reciben una
    // contraseña temporal; en un caso real se enviaría un correo de invitación.
    let password_hash = User::hash_password("changeme123").unwrap_or_default();

    let result = sqlx::query("INSERT INTO users (name, email, password_hash) VALUES (?, ?, ?)")
        .bind(&form.name)
        .bind(&form.email)
        .bind(&password_hash)
        .execute(&pool)
        .await;

    if let Err(sqlx::Error::Database(db_err)) = &result {
        if db_err.is_unique_violation() {
            return Err(UserFormTemplate {
                user: Some(blank_user(0, &form)),
                error: Some("Ya existe un usuario con ese correo.".to_string()),
            });
        }
    }
    if result.is_err() {
        return Err(UserFormTemplate {
            user: None,
            error: Some("No se pudo crear el usuario.".to_string()),
        });
    }

    Ok(Redirect::to("/users"))
}

pub async fn edit(
    State(pool): State<AnyPool>,
    Path(id): Path<i64>,
) -> Result<UserFormTemplate, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, password_hash FROM users WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(UserFormTemplate {
        user: Some(user),
        error: None,
    })
}

pub async fn update(
    State(pool): State<AnyPool>,
    Path(id): Path<i64>,
    Form(form): Form<UserForm>,
) -> Result<Redirect, UserFormTemplate> {
    let result = sqlx::query("UPDATE users SET name = ?, email = ? WHERE id = ?")
        .bind(&form.name)
        .bind(&form.email)
        .bind(id)
        .execute(&pool)
        .await;

    if let Err(sqlx::Error::Database(db_err)) = &result {
        if db_err.is_unique_violation() {
            return Err(UserFormTemplate {
                user: Some(blank_user(id, &form)),
                error: Some("Ya existe un usuario con ese correo.".to_string()),
            });
        }
    }
    if result.is_err() {
        return Err(UserFormTemplate {
            user: Some(blank_user(id, &form)),
            error: Some("No se pudo actualizar el usuario.".to_string()),
        });
    }

    Ok(Redirect::to("/users"))
}

pub async fn destroy(
    State(pool): State<AnyPool>,
    Path(id): Path<i64>,
) -> Result<&'static str, AppError> {
    sqlx::query("DELETE FROM users WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;

    Ok("")
}
