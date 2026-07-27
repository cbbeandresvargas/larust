use askama::Template;
use axum::{extract::State, response::Redirect, Form};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;

use crate::db::{new_id, AppState};
use crate::middleware::auth::SESSION_COOKIE;
use crate::models::User;

const SESSION_DURATION_SECS: i64 = 60 * 60 * 24 * 7; // 7 días

#[derive(Template)]
#[template(path = "auth/login.html")]
pub struct LoginTemplate {
    pub error: Option<String>,
}

#[derive(Template)]
#[template(path = "auth/register.html")]
pub struct RegisterTemplate {
    pub error: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterForm {
    pub name: String,
    pub email: String,
    pub password: String,
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

async fn create_session(state: &AppState, user_id: &str) -> Result<String, sqlx::Error> {
    let session_id = new_id();
    let expires_at = now_unix() + SESSION_DURATION_SECS;
    sqlx::query(&state.sql("INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, ?)"))
        .bind(&session_id)
        .bind(user_id)
        .bind(expires_at)
        .execute(&state.pool)
        .await?;
    Ok(session_id)
}

fn session_cookie(session_id: String) -> Cookie<'static> {
    Cookie::build((SESSION_COOKIE, session_id))
        .path("/")
        .http_only(true)
        .build()
}

pub async fn show_login() -> LoginTemplate {
    LoginTemplate { error: None }
}

pub async fn show_register() -> RegisterTemplate {
    RegisterTemplate { error: None }
}

pub async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Result<(CookieJar, Redirect), LoginTemplate> {
    let user = sqlx::query_as::<_, User>(&state.sql(
        "SELECT id, name, email, password_hash FROM users WHERE email = ?",
    ))
    .bind(&form.email)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten()
    .filter(|u| u.verify_password(&form.password));

    let Some(user) = user else {
        return Err(LoginTemplate {
            error: Some("Correo o contraseña incorrectos.".to_string()),
        });
    };

    let session_id = create_session(&state, &user.id).await.map_err(|_| LoginTemplate {
        error: Some("Ocurrió un error interno, intenta de nuevo.".to_string()),
    })?;

    Ok((jar.add(session_cookie(session_id)), Redirect::to("/users")))
}

pub async fn register(
    State(state): State<AppState>,
    jar: CookieJar,
    Form(form): Form<RegisterForm>,
) -> Result<(CookieJar, Redirect), RegisterTemplate> {
    let password_hash = User::hash_password(&form.password).map_err(|_| RegisterTemplate {
        error: Some("No se pudo procesar la contraseña.".to_string()),
    })?;

    let user_id = new_id();
    let result = sqlx::query(&state.sql(
        "INSERT INTO users (id, name, email, password_hash) VALUES (?, ?, ?, ?)",
    ))
    .bind(&user_id)
    .bind(&form.name)
    .bind(&form.email)
    .bind(&password_hash)
    .execute(&state.pool)
    .await;

    if let Err(sqlx::Error::Database(db_err)) = &result {
        if db_err.is_unique_violation() {
            return Err(RegisterTemplate {
                error: Some("Ese correo ya está registrado.".to_string()),
            });
        }
    }
    if result.is_err() {
        return Err(RegisterTemplate {
            error: Some("No se pudo crear la cuenta.".to_string()),
        });
    }

    let session_id = create_session(&state, &user_id).await.map_err(|_| RegisterTemplate {
        error: Some("Cuenta creada. Inicia sesión manualmente.".to_string()),
    })?;

    Ok((jar.add(session_cookie(session_id)), Redirect::to("/users")))
}

pub async fn logout(State(state): State<AppState>, jar: CookieJar) -> (CookieJar, Redirect) {
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        let _ = sqlx::query(&state.sql("DELETE FROM sessions WHERE id = ?"))
            .bind(cookie.value())
            .execute(&state.pool)
            .await;
    }

    (jar.remove(Cookie::from(SESSION_COOKIE)), Redirect::to("/login"))
}
