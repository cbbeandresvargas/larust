use askama::Template;
use axum::{
    extract::{Multipart, State},
    response::Redirect,
    Extension,
};
use sqlx::AnyPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::{Upload, User};

const UPLOAD_DIR: &str = "static/uploads";

#[derive(Template)]
#[template(path = "uploads/index.html")]
pub struct UploadsTemplate {
    pub uploads: Vec<Upload>,
}

pub async fn index(State(pool): State<AnyPool>) -> Result<UploadsTemplate, AppError> {
    let uploads = sqlx::query_as::<_, Upload>(
        "SELECT id, user_id, filename, original_name, content_type, size, created_at \
         FROM uploads ORDER BY id DESC",
    )
    .fetch_all(&pool)
    .await?;

    Ok(UploadsTemplate { uploads })
}

/// Guarda cada campo de archivo del formulario en `static/uploads/` con un
/// nombre generado (UUID) y registra el archivo en la base de datos.
/// El nombre original se reduce a su componente base (`file_name()`) antes de
/// usarlo para evitar que un valor como `../../etc/passwd` escriba fuera del
/// directorio de subidas.
pub async fn store(
    Extension(user): Extension<User>,
    State(pool): State<AnyPool>,
    mut multipart: Multipart,
) -> Result<Redirect, AppError> {
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AppError::BadRequest("Formulario de subida inválido.".to_string()))?
    {
        if field.name() != Some("file") {
            continue;
        }

        let original_name = field.file_name().unwrap_or("archivo").to_string();
        let safe_name = std::path::Path::new(&original_name)
            .file_name()
            .and_then(|n| n.to_str())
            .filter(|n| !n.is_empty())
            .unwrap_or("archivo")
            .to_string();
        let content_type = field.content_type().map(|c| c.to_string());

        let data = field
            .bytes()
            .await
            .map_err(|_| AppError::BadRequest("No se pudo leer el archivo.".to_string()))?;

        let stored_name = format!("{}-{}", Uuid::new_v4(), safe_name);
        let path = format!("{}/{}", UPLOAD_DIR, stored_name);
        tokio::fs::write(&path, &data).await?;

        sqlx::query(
            "INSERT INTO uploads (user_id, filename, original_name, content_type, size) \
             VALUES (?, ?, ?, ?, ?)",
        )
        .bind(user.id)
        .bind(&stored_name)
        .bind(&original_name)
        .bind(content_type)
        .bind(data.len() as i64)
        .execute(&pool)
        .await?;
    }

    Ok(Redirect::to("/uploads"))
}
