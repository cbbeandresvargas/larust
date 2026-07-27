use askama::Template;
use axum::{
    extract::{Multipart, State},
    response::Redirect,
    Extension,
};

use crate::db::{new_id, AppState, Insertable, Model};
use crate::error::AppError;
use crate::models::{Upload, User};

const UPLOAD_DIR: &str = "static/uploads";

#[derive(Template)]
#[template(path = "uploads/index.html")]
pub struct UploadsTemplate {
    pub uploads: Vec<Upload>,
}

pub async fn index(State(state): State<AppState>) -> Result<UploadsTemplate, AppError> {
    let mut uploads = Upload::all(&state).await?;
    uploads.reverse(); // más recientes primero (Model::all ordena por id ascendente)

    Ok(UploadsTemplate { uploads })
}

/// Guarda cada campo de archivo del formulario en `static/uploads/` con un
/// nombre generado (UUIDv7, el mismo que se usa como id de la fila) y
/// registra el archivo en la base de datos.
/// El nombre original se reduce a su componente base (`file_name()`) antes de
/// usarlo para evitar que un valor como `../../etc/passwd` escriba fuera del
/// directorio de subidas.
pub async fn store(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
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

        let id = new_id();
        let stored_name = format!("{}-{}", id, safe_name);
        let path = format!("{}/{}", UPLOAD_DIR, stored_name);
        tokio::fs::write(&path, &data).await?;

        let upload = Upload {
            id,
            user_id: Some(user.id.clone()),
            filename: stored_name,
            original_name,
            content_type,
            size: data.len() as i64,
            created_at: String::new(), // lo asigna DEFAULT CURRENT_TIMESTAMP al insertar
        };
        upload.create(&state).await?;
    }

    Ok(Redirect::to("/uploads"))
}
