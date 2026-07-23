# Documentación de Larust 🦀

Bienvenido a la documentación de **Larust**. En esta guía encontrarás instrucciones detalladas paso a paso sobre cómo desarrollar utilizando la arquitectura MVC inspirada en Laravel en tu proyecto de Rust con Axum, Askama y SQLx.

## 🗂️ Índice de Contenidos

1. 🏛️ **[Arquitectura](file:///c:/GH/larust/docs/architecture.md)**: Estructura del ciclo de vida de peticiones, diseño MVC y gestión de estado.
2. 🎛️ **[Controladores](file:///c:/GH/larust/docs/controllers.md)**: Cómo crear nuevos controladores y gestionar las peticiones y respuestas.
3. 🔒 **[Middlewares](file:///c:/GH/larust/docs/middleware.md)**: Cómo interceptar peticiones HTTP para autenticación, registro de logs, etc.
4. 📦 **[Modelos](file:///c:/GH/larust/docs/models.md)**: Cómo definir estructuras de datos seguras y mapear tablas de bases de datos.
5. 💾 **[Base de Datos](file:///c:/GH/larust/docs/database.md)**: Cómo editar la base de datos, ejecutar consultas SQLx, alternar entre SQLite/PostgreSQL y usar migraciones.
6. 🎨 **[Vistas e HTMX](file:///c:/GH/larust/docs/views.md)**: Cómo crear plantillas HTML dinámicas y reactivas con Askama y HTMX.

---

## 🚀 Filosofía de Desarrollo

Larust sigue el principio de **"Convención sobre Configuración"** dentro de lo posible en Rust:
- Las **vistas** se definen en la carpeta `templates/` y se compilan en tiempo de compilación.
- Los **controladores** encapsulan la lógica de negocio y se ubican en `src/controllers/`.
- Los **modelos** representan las entidades de datos y se ubican en `src/models/`.
- Las **rutas** se configuran de forma centralizada en `src/routes.rs`.
