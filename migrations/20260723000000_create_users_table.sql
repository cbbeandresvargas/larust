-- Migración inicial: Crear tabla de usuarios
-- Esta tabla funcionará en SQLite de forma preconfigurada.
-- Si cambias a Postgres, recuerda usar SERIAL o BIGSERIAL para la clave primaria autoincremental.

CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE
);
