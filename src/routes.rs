use axum::{
    middleware,
    routing::get,
    Router,
};
use sqlx::AnyPool;

use crate::controllers::{auth_controller, home_controller, upload_controller, user_controller};
use crate::middleware::auth::require_auth;

/// Rutas de la aplicación web que renderizan vistas HTML (Askama).
///
/// `/users` y `/uploads` requieren sesión activa: se agrupan bajo un router
/// separado con `require_auth` aplicado vía `.route_layer`, que se evalúa
/// después de que Axum ya resolvió cuál handler atiende la petición.
pub fn web_routes(pool: AnyPool) -> Router<AnyPool> {
    let protected = Router::new()
        .route(
            "/users",
            get(user_controller::index).post(user_controller::create),
        )
        .route("/users/new", get(user_controller::new))
        .route("/users/:id/edit", get(user_controller::edit))
        .route(
            "/users/:id",
            axum::routing::post(user_controller::update).delete(user_controller::destroy),
        )
        .route(
            "/uploads",
            get(upload_controller::index).post(upload_controller::store),
        )
        .route("/logout", axum::routing::post(auth_controller::logout))
        .route_layer(middleware::from_fn_with_state(pool, require_auth));

    Router::new()
        .route("/", get(home_controller::index))
        .route(
            "/login",
            get(auth_controller::show_login).post(auth_controller::login),
        )
        .route(
            "/register",
            get(auth_controller::show_register).post(auth_controller::register),
        )
        .merge(protected)
}

/// Rutas de API y endpoints para HTMX
pub fn api_routes() -> Router<AnyPool> {
    Router::new().route("/ping", get(home_controller::ping))
}
