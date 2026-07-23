-- Añade el campo de contraseña (hash bcrypt) a los usuarios existentes.
ALTER TABLE users ADD COLUMN password_hash VARCHAR(255) NOT NULL DEFAULT '';
