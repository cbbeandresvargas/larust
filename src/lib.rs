pub mod controllers;
pub mod db;
pub mod error;
pub mod middleware;
pub mod models;
pub mod routes;

use axum::Router;
use tower_http::services::ServeDir;

pub use db::AppState;

/// Construye el router completo de la aplicación a partir de un `AppState`
/// ya armado (pool conectado + bandera de motor). Se expone como función de
/// biblioteca (en vez de vivir sólo en `main.rs`) para que los tests de
/// integración en `tests/` puedan levantar exactamente el mismo árbol de
/// rutas que corre en producción.
pub fn build_app(state: AppState) -> Router {
    Router::new()
        .merge(routes::web_routes(state.clone()))
        .nest("/api", routes::api_routes())
        .nest_service("/static", ServeDir::new("static"))
        .fallback(error::not_found)
        .with_state(state)
}
