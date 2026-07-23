use askama::Template;
use axum::{
    http::StatusCode,
    response::Response,
};

#[derive(Template)]
#[template(path = "errors/404.html")]
pub struct NotFoundTemplate;

#[derive(Template)]
#[template(path = "errors/500.html")]
pub struct ServerErrorTemplate {
    pub message: String,
}

/// Error unificado de la aplicación: centraliza el mapeo de fallos internos
/// (DB, E/S) a respuestas HTTP en vez de dejar que cada controlador use
/// `unwrap_or_default()` y esconda el error real.
pub enum AppError {
    NotFound,
    BadRequest(String),
    Internal(String),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        eprintln!("🦀 Error de base de datos: {}", err);
        AppError::Internal("Ocurrió un error con la base de datos.".to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        eprintln!("🦀 Error de E/S: {}", err);
        AppError::Internal("Ocurrió un error al procesar el archivo.".to_string())
    }
}

impl axum::response::IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => (StatusCode::NOT_FOUND, NotFoundTemplate).into_response(),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg).into_response(),
            AppError::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, ServerErrorTemplate { message: msg }).into_response()
            }
        }
    }
}

pub async fn not_found() -> impl axum::response::IntoResponse {
    (StatusCode::NOT_FOUND, NotFoundTemplate)
}
