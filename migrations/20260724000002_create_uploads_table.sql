-- Registro de archivos subidos por los usuarios.
-- Si cambias a Postgres, recuerda usar SERIAL o BIGSERIAL para la clave primaria autoincremental.
-- created_at usa VARCHAR en vez de TIMESTAMP: el driver agnóstico sqlx::Any
-- no sabe mapear el tipo TIMESTAMP/DATETIME de SQLite al decodificar como
-- String, aunque el valor almacenado (vía CURRENT_TIMESTAMP) sea texto.
CREATE TABLE IF NOT EXISTS uploads (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER REFERENCES users(id),
    filename VARCHAR(255) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    content_type VARCHAR(100),
    size INTEGER NOT NULL,
    created_at VARCHAR(32) NOT NULL DEFAULT CURRENT_TIMESTAMP
);
