# Base de Datos en Larust 💾

Larust utiliza **SQLx** con un pool de conexiones dinámico (`sqlx::AnyPool`), lo que te permite cambiar el motor de base de datos entre SQLite y PostgreSQL modificando únicamente tu archivo `.env`.

---

## ⚙️ Configuración (.env)

Asegúrate de configurar la cadena de conexión correcta:

```ini
# Para SQLite:
DATABASE_URL=sqlite://database.sqlite

# Para PostgreSQL:
# DATABASE_URL=postgres://usuario:password@localhost:5432/larust_db
```

---

## 🛠️ Cómo Editar tu Base de Datos (Tablas y Columnas)

Larust viene configurado con un sistema de **migraciones automáticas** que detecta y aplica cambios estructurales en tu base de datos cada vez que inicia la aplicación.

### Paso 1: Crear un nuevo archivo de migración (.sql)
Crea tus archivos SQL dentro del directorio raíz `migrations/` usando un identificador numérico o timestamp secuencial al inicio del nombre del archivo.

Ejemplo: `migrations/20260723000000_create_users_table.sql`

Escribe tu sentencia DDL estándar dentro del archivo:
```sql
CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE
);
```

> [!NOTE]
> Dado que Larust es agnóstico a la base de datos, si decides cambiar de SQLite a PostgreSQL en tu archivo `.env`, asegúrate de actualizar la sintaxis de la clave primaria a `SERIAL PRIMARY KEY` o `GENERATED ALWAYS AS IDENTITY`.

### Paso 2: Ejecución automática
No necesitas correr ningún comando manual para migrar tu base de datos local en desarrollo. Al ejecutar el servidor con:
```bash
npm run dev
```

Larust compilará los archivos SQL en el binario y ejecutará de forma automática el runner interno de SQLx (`sqlx::migrate!("./migrations").run(&pool).await`) aplicando las nuevas tablas o campos estructurados de forma inmediata.

---

### Opción Avanzada: Migraciones manuales usando `sqlx-cli`

Si prefieres mayor control (como deshacer migraciones o crearlas con el generador de comandos):

1. **Instalar la herramienta de comandos:**
   ```bash
   cargo install sqlx-cli --no-default-features --features native-tls,sqlite,postgres
   ```

2. **Crear una nueva migración con la CLI:**
   ```bash
   sqlx migrate add crear_tabla_productos
   ```
   Esto autogenerará el archivo `.sql` vacío con el timestamp correcto dentro de tu carpeta `migrations/`.

3. **Ejecutar las migraciones pendientes manualmente:**
   ```bash
   sqlx migrate run
   ```

---

## 🔍 Ejecución de Consultas con SQLx

SQLx te permite ejecutar consultas con o sin mapeo automático:

### 1. Consultar múltiples registros mapeados a un Modelo (`query_as`)
```rust
let products = sqlx::query_as::<_, Product>("SELECT id, name, price, stock FROM products")
    .fetch_all(&db)
    .await?;
```

### 2. Consultar un único registro por ID
```rust
let product = sqlx::query_as::<_, Product>("SELECT id, name, price, stock FROM products WHERE id = ?")
    .bind(product_id) // SQLx se encarga de prevenir inyección SQL
    .fetch_one(&db)
    .await?;
```

### 3. Insertar, Actualizar o Eliminar registros (`execute`)
```rust
sqlx::query("INSERT INTO products (name, price, stock) VALUES (?, ?, ?)")
    .bind("Nuevo Laptop")
    .bind(899.99)
    .bind(10)
    .execute(&db)
    .await?;
```

> [!TIP]
> Al usar el driver agnóstico `sqlx::Any`, el marcador de posición para parámetros en las consultas SQL es siempre el signo de interrogación (`?`), tanto para SQLite como para PostgreSQL.
