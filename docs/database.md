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

> [!TIP]
> `cargo make:migration create_products_table` genera el archivo con el
> timestamp correcto y el esqueleto de abajo automáticamente — ver "CLI de
> scaffolding" en `CLAUDE.md`.

### Paso 1: Crear un nuevo archivo de migración (.sql)
Crea tus archivos SQL dentro del directorio raíz `migrations/` usando un identificador numérico o timestamp secuencial al inicio del nombre del archivo.

Ejemplo: `migrations/20260723000000_create_users_table.sql`

Escribe tu sentencia DDL estándar dentro del archivo:
```sql
CREATE TABLE IF NOT EXISTS products (
    id VARCHAR(36) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    price INTEGER NOT NULL
);
```

> [!NOTE]
> La clave primaria es un `VARCHAR(36)`, no un entero autoincremental: SQLite
> (`AUTOINCREMENT`) y Postgres (`SERIAL`/`GENERATED ALWAYS AS IDENTITY`) no
> comparten sintaxis para eso, así que en Larust el `id` es un UUIDv7 (texto)
> generado por la aplicación con `larust::db::new_id()` antes del `INSERT` —
> ver `src/models/user.rs` y `src/controllers/user_controller.rs::create` para
> el patrón completo. Con eso, la misma migración sirve para ambos motores.

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

SQLx te permite ejecutar consultas con o sin mapeo automático. Todos los
handlers reciben `State<AppState>` (`src/db.rs`), no `State<AnyPool>`
directamente: `AppState` trae el pool (`state.pool`) más el helper
`state.sql(...)`.

### 1. Consultar múltiples registros mapeados a un Modelo (`query_as`)
```rust
let products = sqlx::query_as::<_, Product>("SELECT id, name, price, stock FROM products")
    .fetch_all(&state.pool)
    .await?;
```

### 2. Consultar un único registro por ID
```rust
let product = sqlx::query_as::<_, Product>(&state.sql("SELECT id, name, price, stock FROM products WHERE id = ?"))
    .bind(product_id) // SQLx se encarga de prevenir inyección SQL
    .fetch_one(&state.pool)
    .await?;
```

### 3. Insertar, Actualizar o Eliminar registros (`execute`)
```rust
let id = larust::db::new_id(); // UUIDv7 generado en la app, ver nota sobre migraciones arriba
sqlx::query(&state.sql("INSERT INTO products (id, name, price, stock) VALUES (?, ?, ?, ?)"))
    .bind(&id)
    .bind("Nuevo Laptop")
    .bind(899.99)
    .bind(10)
    .execute(&state.pool)
    .await?;
```

> [!IMPORTANT]
> **Siempre escribe `?` como placeholder, pero pásalo por `state.sql(...)`
> antes de dárselo a `sqlx::query`/`query_as`.** A pesar de lo que sugeriría
> el nombre "driver agnóstico", `sqlx::Any` **no** traduce `?` a `$1, $2, ...`
> por su cuenta: SQLite acepta `?` de forma nativa, pero Postgres lo rechaza
> con un error de sintaxis. `state.sql()` (`src/db.rs`) hace esa traducción
> en tiempo de ejecución según el motor conectado — sin ella, cualquier
> consulta con `.bind()` falla contra Postgres.
