-- Sesiones de autenticación: un token opaco por sesión de usuario.
CREATE TABLE IF NOT EXISTS sessions (
    id VARCHAR(64) PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id),
    expires_at BIGINT NOT NULL
);
