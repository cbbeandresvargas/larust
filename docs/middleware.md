# Middlewares en Larust 🔒

Los middlewares te permiten interceptar las peticiones HTTP antes de que lleguen a tus controladores, o modificar la respuesta antes de enviarla de vuelta al usuario. Son útiles para:
- Autenticación y Autorización.
- Logging y métricas.
- Manejo de cabeceras CORS o seguridad.

---

## 🛠️ Cómo crear un Middleware usando Axum

Axum permite crear middlewares de forma sencilla utilizando la función `axum::middleware::from_fn`.

### Paso 1: Crear la función del Middleware
Crea una carpeta `src/middleware/` (o coloca la función en un módulo adecuado). Por ejemplo, creamos `src/middleware/auth.rs`:

```rust
use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
    http::StatusCode,
};

pub async fn check_auth(
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // 1. Lógica previa: Leer cabeceras de la petición
    if let Some(auth_header) = req.headers().get("Authorization") {
        if auth_header == "Bearer token-secreto" {
            // El token es válido, continúa al siguiente handler o middleware
            let response = next.run(req).await;
            return Ok(response);
        }
    }

    // 2. Si falla la autenticación, se aborta y retorna un error 401 Unauthorized
    Err(StatusCode::UNAUTHORIZED)
}
```

### Paso 2: Registrar y aplicar el Middleware en `src/main.rs` o `src/routes.rs`

Para aplicar el middleware a un grupo de rutas, utiliza la función `.route_layer` de Axum:

```rust
// src/routes.rs
use axum::{routing::get, Router, middleware};
use sqlx::AnyPool;
use crate::controllers::{home_controller, user_controller};
mod middleware_auth {
    // Importa el middleware creado
}

pub fn web_routes() -> Router<AnyPool> {
    // Rutas protegidas
    let rutas_protegidas = Router::new()
        .route("/users", get(user_controller::index))
        // Aplica el middleware únicamente a estas rutas
        .route_layer(middleware::from_fn(check_auth));

    Router::new()
        .route("/", get(home_controller::index))
        .merge(rutas_protegidas)
}
```

> [!IMPORTANT]
> Los layers en Axum se ejecutan de **abajo hacia arriba** (orden inverso de declaración). Coloca `.route_layer` después de definir las rutas que deseas proteger.
