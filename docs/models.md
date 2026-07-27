# Modelos en Larust 📦

Los modelos representan la estructura de los datos de la aplicación y la lógica de negocio asociada. En Larust, los modelos interactúan con la base de datos a través de **SQLx**.

---

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

## 🚀 Uso del Modelo en un Controlador

Para usar el modelo e interactuar con la base de datos desde tu controlador, puedes hacer lo siguiente:

```rust
use axum::extract::State;
use larust::db::AppState;
use crate::models::Product;

pub async fn list_products(State(state): State<AppState>) -> Result<axum::Json<Vec<Product>>, String> {
    let products = sqlx::query_as::<_, Product>("SELECT id, name, description, price, stock FROM products")
        .fetch_all(&state.pool)
        .await
        .map_err(|e| e.to_string())?;

    Ok(axum::Json(products))
}
```
