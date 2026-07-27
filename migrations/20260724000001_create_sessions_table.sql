-- Sesiones de autenticación: un token opaco (UUIDv7) por sesión de usuario.
CREATE TABLE IF NOT EXISTS sessions (
    id VARCHAR(36) PRIMARY KEY,
    user_id VARCHAR(36) NOT NULL REFERENCES users(id),
    expires_at BIGINT NOT NULL
);
