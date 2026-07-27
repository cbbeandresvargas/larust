-- Migración inicial: Crear tabla de usuarios.
-- id es un UUIDv7 (texto) generado en la aplicación (Uuid::now_v7()), no un
-- autoincremental de la base de datos: SQLite y Postgres no comparten
-- sintaxis para claves autoincrementales (AUTOINCREMENT vs. SERIAL) ni ancho
-- de entero, mientras que un VARCHAR es idéntico en ambos motores. UUIDv7 es
-- ordenable por tiempo de creación, a diferencia de UUIDv4, así que los IDs
-- siguen siendo aproximadamente secuenciales para índices y depuración.
CREATE TABLE IF NOT EXISTS users (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE
);
