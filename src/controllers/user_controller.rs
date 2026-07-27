use askama::Template;
use axum::{
    extract::{Path, State},
    response::Redirect,
    Form,
};
use serde::Deserialize;

use crate::db::{new_id, AppState};
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

fn blank_user(id: String, form: &UserForm) -> User {
    User {
        id,
        name: form.name.clone(),
        email: form.email.clone(),
        password_hash: String::new(),
    }
}

pub async fn index(State(state): State<AppState>) -> Result<UsersTemplate, AppError> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, name, email, password_hash FROM users ORDER BY id",
    )
    .fetch_all(&state.pool)
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
    State(state): State<AppState>,
    Form(form): Form<UserForm>,
) -> Result<Redirect, UserFormTemplate> {
    // Los usuarios creados desde este formulario de administración reciben una
    // contraseña temporal; en un caso real se enviaría un correo de invitación.
    let password_hash = User::hash_password("changeme123").unwrap_or_default();
    let id = new_id();

    let result = sqlx::query(&state.sql(
        "INSERT INTO users (id, name, email, password_hash) VALUES (?, ?, ?, ?)",
    ))
    .bind(&id)
    .bind(&form.name)
    .bind(&form.email)
    .bind(&password_hash)
    .execute(&state.pool)
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
            user: None,
            error: Some("No se pudo crear el usuario.".to_string()),
        });
    }

    Ok(Redirect::to("/users"))
}

pub async fn edit(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<UserFormTemplate, AppError> {
    let user = sqlx::query_as::<_, User>(&state.sql(
        "SELECT id, name, email, password_hash FROM users WHERE id = ?",
    ))
    .bind(id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

    Ok(UserFormTemplate {
        user: Some(user),
        error: None,
    })
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Form(form): Form<UserForm>,
) -> Result<Redirect, UserFormTemplate> {
    let result = sqlx::query(&state.sql("UPDATE users SET name = ?, email = ? WHERE id = ?"))
        .bind(&form.name)
        .bind(&form.email)
        .bind(&id)
        .execute(&state.pool)
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
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<&'static str, AppError> {
    sqlx::query(&state.sql("DELETE FROM users WHERE id = ?"))
        .bind(id)
        .execute(&state.pool)
        .await?;

    Ok("")
}
