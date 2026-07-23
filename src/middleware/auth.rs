use axum::{
    extract::{Request, State},
    middleware::Next,
    response::{Redirect, Response},
};
use axum_extra::extract::cookie::CookieJar;
use sqlx::AnyPool;

use crate::models::{Session, User};

pub const SESSION_COOKIE: &str = "session_id";

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

/// Middleware de autenticación: exige una sesión válida (cookie `session_id`
/// respaldada por la tabla `sessions`) y, de tenerla, inyecta el `User`
/// autenticado en las extensiones de la petición para que los handlers lo
/// extraigan con `Extension<User>`. Si no hay sesión válida, redirige a /login.
pub async fn require_auth(
    State(pool): State<AnyPool>,
    jar: CookieJar,
    mut req: Request,
    next: Next,
) -> Result<Response, Redirect> {
    let Some(cookie) = jar.get(SESSION_COOKIE) else {
        return Err(Redirect::to("/login"));
    };

    let session = sqlx::query_as::<_, Session>(
        "SELECT id, user_id, expires_at FROM sessions WHERE id = ?",
    )
    .bind(cookie.value())
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    let Some(session) = session else {
        return Err(Redirect::to("/login"));
    };

    if session.expires_at < now_unix() {
        return Err(Redirect::to("/login"));
    }

    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, email, password_hash FROM users WHERE id = ?",
    )
    .bind(session.user_id)
    .fetch_optional(&pool)
    .await
    .ok()
    .flatten();

    let Some(user) = user else {
        return Err(Redirect::to("/login"));
    };

    req.extensions_mut().insert(user);
    Ok(next.run(req).await)
}
