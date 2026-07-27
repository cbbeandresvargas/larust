# Controladores en Larust 🎛️

Los controladores en Larust encapsulan la lógica de manejo de solicitudes HTTP (Handlers) y devuelven respuestas HTTP, siguiendo la estructura MVC.

---

## 🛠️ Cómo crear un controlador paso a paso

### Paso 1: Crear el archivo del controlador
Crea un nuevo archivo en `src/controllers/`. Por ejemplo, `src/controllers/product_controller.rs`:

```rust
use axum::extract::{Path, State};
use askama::Template;
use askama_axum::IntoResponse;
use sqlx::AnyPool;

// 1. Define la plantilla de Askama si va a renderizar una vista HTML
#[derive(Template)]
#[template(path = "products/show.html")]
pub struct ProductTemplate {
    pub name: String,
    pub price: f64,
}

// 2. Handler para renderizar una vista HTML
pub async fn show(Path(id): Path<String>) -> impl IntoResponse {
    // Aquí iría la consulta a la base de datos
    ProductTemplate {
        name: format!("Producto #{}", id),
        price: 99.99,
    }
}

// 3. Handler para retornar un JSON o texto simple (útil para APIs o HTMX)
pub async fn api_status() -> &'static str {
    "Ok"
}
```

### Paso 2: Registrar el controlador en `src/controllers/mod.rs`
Debes exponer el nuevo módulo en el archivo principal de controladores para que sea accesible desde las rutas:

```rust
// src/controllers/mod.rs
pub mod home_controller;
pub mod user_controller;
pub mod product_controller; // <-- Añade esta línea
```

### Paso 3: Asignar una ruta en `src/routes.rs`
Importa el controlador y mapea la ruta correspondiente en el enrutador de Axum:

```rust
// src/routes.rs
use crate::controllers::{home_controller, user_controller, product_controller};
use crate::db::AppState;

pub fn web_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(home_controller::index))
        .route("/users", get(user_controller::index))
        .route("/products/:id", get(product_controller::show)) // <-- Ruta Web
}
```

---

## 📥 Extrayendo datos en los Handlers

Axum proporciona extractores muy potentes que se pasan como argumentos a las funciones del controlador:

- **Estado global (`State`)**: `State(state): State<AppState>` para acceder a `state.pool` (el `AnyPool`) y a `state.sql(...)` (ver docs/database.md sobre por qué hace falta para Postgres).
- **Parámetros de ruta (`Path`)**: `Path(id): Path<String>` para rutas como `/users/:id` — los IDs son UUIDv7 en texto, no enteros.
- **Query Params (`Query`)**: `Query(params): Query<MyStruct>` para `/search?q=rust`.
- **Formularios (`Form`)**: `Form(input): Form<MyFormStruct>` para procesar envíos `POST`.
- **Cuerpo JSON (`Json`)**: `Json(payload): Json<MyStruct>` para peticiones de API REST.
