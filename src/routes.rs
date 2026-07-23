use axum::{routing::get, Router};
use sqlx::AnyPool;
use crate::controllers::{home_controller, user_controller};

/// Rutas de la aplicación web que renderizan vistas HTML (Askama)
pub fn web_routes() -> Router<AnyPool> {
    Router::new()
        .route("/", get(home_controller::index))
        .route("/users", get(user_controller::index))
}

/// Rutas de API y endpoints para HTMX
pub fn api_routes() -> Router<AnyPool> {
    Router::new()
        .route("/ping", get(home_controller::ping))
}
