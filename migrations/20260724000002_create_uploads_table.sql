-- Registro de archivos subidos por los usuarios.
-- id/user_id son UUIDv7 (texto), igual que en users (ver esa migración).
-- created_at usa VARCHAR en vez de TIMESTAMP: el driver agnóstico sqlx::Any
-- no sabe mapear el tipo TIMESTAMP/DATETIME de SQLite al decodificar como
-- String, aunque el valor almacenado (vía CURRENT_TIMESTAMP) sea texto.
CREATE TABLE IF NOT EXISTS uploads (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) REFERENCES users(id),
    filename VARCHAR(255) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    content_type VARCHAR(100),
    size BIGINT NOT NULL,
    created_at VARCHAR(32) NOT NULL DEFAULT CURRENT_TIMESTAMP
);
