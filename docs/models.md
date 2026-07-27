# Modelos en Larust 📦

Los modelos representan la estructura de los datos de la aplicación y la lógica de negocio asociada. En Larust, los modelos interactúan con la base de datos a través de **SQLx**.

---

> [!TIP]
> `cargo make:model Product` genera automáticamente el archivo del modelo y
> lo registra en `mod.rs` — ver "CLI de scaffolding" en `CLAUDE.md`. Lo de
> abajo explica el patrón que ese comando reproduce.

## 🛠️ Cómo crear un modelo paso a paso

### Paso 1: Crear la estructura del modelo
Crea un nuevo archivo en `src/models/`. Por ejemplo, `src/models/product.rs`:

```rust
use serde::{Serialize, Deserialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: String, // UUIDv7 generado en la app, no un entero autoincremental (ver docs/database.md)
    pub name: String,
    pub description: Option<String>, // Los valores NULL en DB mapean a Option<T>
    pub price: f64,
    pub stock: i32,
}

impl Product {
    // Puedes encapsular lógica del modelo aquí
    pub fn is_in_stock(&self) -> bool {
        self.stock > 0
    }
}
```

### Paso 2: Registrar el modelo en `src/models/mod.rs`
Exporta el modelo para que esté disponible en toda la aplicación:

```rust
// src/models/mod.rs
pub mod user;
pub mod product; // <-- Registrar submódulo

pub use user::User;
pub use product::Product; // <-- Re-exportar estructura
```

---

## 🧰 CRUD genérico con `Model`/`Insertable` (`src/db.rs`)

No es un ORM (sin relaciones ni query builder), pero elimina el `SELECT`/`DELETE ... WHERE id = ?` repetido en cada controlador. `#[derive(FromRow)]` mapea columnas **por nombre**, así que un `SELECT *` genérico funciona para cualquier struct sin importar el orden de sus campos.

### Lectura y borrado: solo `Model`
```rust
use crate::db::Model;

impl Model for Product {
    const TABLE: &'static str = "products";
}
```
Con eso ya tienes gratis:
```rust
Product::find(&state, &id).await?   // Option<Product>
Product::all(&state).await?         // Vec<Product>
Product::delete(&state, &id).await? // ()
```

### Crear/actualizar: además `Insertable`
Rust no tiene reflexión, así que hace falta decirle al trait qué columnas insertar (sin el `id`, que ya maneja `create` por su cuenta):

```rust
use crate::db::{Insertable, Model, Value};

impl Insertable for Product {
    fn id(&self) -> &str {
        &self.id
    }

    fn fields(&self) -> Vec<(&'static str, Value)> {
        vec![
            ("name", Value::Str(self.name.clone())),
            ("description", Value::OptStr(self.description.clone())),
            // Value solo tiene Str/OptStr/Int hoy — si tu columna es otro
            // tipo (f64, bool...), agrega la variante en src/db.rs.
        ]
    }
}
```
Con eso: `product.create(&state).await?` y `product.update(&state).await?`.

> [!TIP]
> Solo implementa `Insertable` cuando el alta/edición es realmente genérica.
> `User` (`src/models/user.rs`) implementa únicamente `Model`: su `create`/
> `update` real vive a mano en `user_controller.rs`/`auth_controller.rs`
> porque necesita hashear la contraseña y mostrar "correo ya registrado" en
> caso de violación de unicidad — cosas que un trait genérico no puede
> expresar. Usa el enfoque manual (ver `docs/database.md`) para esos casos.

---

## 🚀 Uso del Modelo en un Controlador

Para consultas que no encajan en `Model`/`Insertable` (joins, filtros, agregaciones), sigue usando SQLx directamente:

```rust
use axum::extract::State;
use crate::db::AppState;
use crate::models::Product;

pub async fn list_products(State(state): State<AppState>) -> Result<axum::Json<Vec<Product>>, String> {
    let products = sqlx::query_as::<_, Product>("SELECT id, name, description, price, stock FROM products")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(axum::Json(products))
}
```
