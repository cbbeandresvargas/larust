use askama::Template;
use axum::{extract::State, response::Redirect, Form};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use serde::Deserialize;
use sqlx::AnyPool;
use uuid::Uuid;

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

async fn create_session(pool: &AnyPool, user_id: i64) -> Result<String, sqlx::Error> {
    let session_id = Uuid::new_v4().to_string();
    let expires_at = now_unix() + SESSION_DURATION_SECS;
    sqlx::query("INSERT INTO sessions (id, user_id, expires_at) VALUES (?, ?, ?)")
        .bind(&session_id)
        .bind(user_id)
        .bind(expires_at)
        .execute(pool)
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
    State(pool): State<AnyPool>,
    jar: CookieJar,
    Form(form): Form<LoginForm>,
) -> Result<(CookieJar, Redirect), LoginTemplate> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, password_hash FROM users WHERE email = ?",
    )
    .bind(&form.email)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten()
    .filter(|u| u.verify_password(&form.password));

    let Some(user) = user else {
        return Err(LoginTemplate {
            error: Some("Correo o contraseña incorrectos.".to_string()),
        });
    };

    let session_id = create_session(&pool, user.id).await.map_err(|_| LoginTemplate {
        error: Some("Ocurrió un error interno, intenta de nuevo.".to_string()),
    })?;

    Ok((jar.add(session_cookie(session_id)), Redirect::to("/users")))
}

pub async fn register(
    State(pool): State<AnyPool>,
    jar: CookieJar,
    Form(form): Form<RegisterForm>,
) -> Result<(CookieJar, Redirect), RegisterTemplate> {
    let password_hash = User::hash_password(&form.password).map_err(|_| RegisterTemplate {
        error: Some("No se pudo procesar la contraseña.".to_string()),
    })?;

    let result = sqlx::query("INSERT INTO users (name, email, password_hash) VALUES (?, ?, ?)")
        .bind(&form.name)
        .bind(&form.email)
        .bind(&password_hash)
        .execute(&pool)
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

    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, password_hash FROM users WHERE email = ?",
    )
    .bind(&form.email)
    .fetch_one(&pool)
    .await;

    let Ok(user) = user else {
        return Err(RegisterTemplate {
            error: Some("Cuenta creada, pero no se pudo iniciar sesión automáticamente.".to_string()),
        });
    };

    let session_id = create_session(&pool, user.id).await.map_err(|_| RegisterTemplate {
        error: Some("Cuenta creada. Inicia sesión manualmente.".to_string()),
    })?;

    Ok((jar.add(session_cookie(session_id)), Redirect::to("/users")))
}

pub async fn logout(State(pool): State<AnyPool>, jar: CookieJar) -> (CookieJar, Redirect) {
    if let Some(cookie) = jar.get(SESSION_COOKIE) {
        let _ = sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(cookie.value())
            .execute(&pool)
            .await;
    }

    (jar.remove(Cookie::from(SESSION_COOKIE)), Redirect::to("/login"))
}
