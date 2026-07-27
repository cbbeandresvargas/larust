# Larust 🦀

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust Version](https://img.shields.io/badge/rustc-1.75+-orange.svg)](https://www.rust-lang.org/)
[![Database](https://img.shields.io/badge/database-SQLite%20%7C%20Postgres-blue.svg)](https://github.com/launchbadge/sqlx)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind_CSS-v4-38bdf8.svg)](https://tailwindcss.com/)
[![HTMX](https://img.shields.io/badge/HTMX-v1.9.12-3b82f6.svg)](https://htmx.org/)

**Larust** es un kit de inicio y plantilla de repositorio (GitHub Template) diseñado para ofrecer una experiencia de desarrollo limpia, ergonómica y estructurada basada en la arquitectura **Model-View-Controller (MVC)** de Laravel, combinada con la potencia, seguridad y el rendimiento extremo del ecosistema de **Rust**.

Con Larust, obtienes la elegancia organizativa de Laravel y la velocidad inigualable de Rust.

---

## ✨ Características (Features)

*   **🗂️ Arquitectura MVC Ergonómica:** Estructura limpia y conocida (Rutas, Controladores, Modelos y Vistas) diseñada para que escales tu aplicación sin perder orden.
*   **🛡️ Plantillas Compiladas con Seguridad de Tipos:** Utiliza **Askama** (sintaxis tipo Blade pero compilado de forma segura en Rust). Si cometes un error de tipo en tu vista HTML, tu aplicación no compilará, previniendo fallos en producción.
*   **🔌 Base de Datos Agnóstica (DB Agnostic):** Intercambia dinámicamente entre SQLite y PostgreSQL simplemente cambiando la variable `DATABASE_URL` en tu archivo `.env` vía **SQLx (AnyPool)** sin tocar una sola línea de código en tu aplicación.
*   **🔥 Hot Reloading (Recarga en caliente):** Desarrollo fluido y ágil gracias a la integración y soporte de auto-recompilación al vuelo mediante `cargo-watch`.
*   **🎨 Frontend Ligero y Dinámico:** Integración local de **HTMX** y **Tailwind CSS v4** preconfigurada. Interactividad tipo SPA sin la complejidad de configurar bundlers de JavaScript (Vite, Webpack, etc.).
*   **🌐 Cero Dependencias de Servidor Exterior:** Servidor web asíncrono e integrado de alto rendimiento con **Axum** y el runtime **Tokio**, listo para producción.

---

## 📁 Estructura del Proyecto

```text
larust/
├── .env.example          # Plantilla de variables de entorno
├── .env                  # Variables locales (ignorado por Git)
├── Cargo.toml            # Configuración de dependencias de Rust
├── LICENSE               # Licencia MIT
├── static/               # Archivos estáticos
│   ├── css/
│   │   ├── input.css     # CSS origen con directivas de Tailwind
│   │   └── output.css    # CSS compilado final de Tailwind
│   └── js/
│       └── htmx.min.js   # Librería HTMX local
├── templates/            # Vistas en HTML (Askama)
│   ├── layouts/
│   │   └── base.html     # Plantilla maestra principal
│   ├── home/
│   │   └── index.html    # Vista de la página de inicio
│   └── users/
│       └── index.html    # Vista del listado de usuarios
└── src/                  # Código fuente
    ├── main.rs           # Punto de entrada, conexión DB e inicio de Axum
    ├── routes.rs         # Mapeo de rutas (estilo web.php / api.php)
    ├── controllers/      # Controladores de la aplicación
    │   ├── mod.rs
    │   ├── home_controller.rs
    │   └── user_controller.rs
    └── models/           # Modelos y estructuras de datos
        ├── mod.rs
        └── user.rs
```

---

## ⚙️ Configuración de la Base de Datos

Larust utiliza un pool de conexiones agnóstico (`sqlx::AnyPool`) que se conecta dinámicamente al motor configurado en tu `.env`.

### SQLite (Por defecto)
```ini
DATABASE_URL=sqlite://database.sqlite
```
*(El archivo `database.sqlite` se creará automáticamente en la raíz del proyecto al iniciar la aplicación en desarrollo).*

### PostgreSQL
```ini
DATABASE_URL=postgres://usuario:password@localhost:5432/larust_db
```

---

## 🛠️ Generador de Código (`cargo make:*`)

Larust incluye comandos estilo `php artisan make:...`, listos para usar sin instalar nada aparte (ya vienen configurados en `.cargo/config.toml`):

```bash
cargo make:model Product          # crea src/models/product.rs y lo registra en mod.rs
cargo make:controller Product     # crea el controlador + su vista, y te indica la ruta a agregar
cargo make:migration create_products_table  # crea migrations/<timestamp>_create_products_table.sql
```

El código generado compila de inmediato (solo referencia el campo `id`); los comentarios `// TODO:` marcan dónde agregar tus columnas reales. Agregar la ruta en `src/routes.rs` queda manual a propósito, ya que implica decidir el verbo HTTP y si la ruta requiere sesión.

---

## 🚀 Desarrollo Local con Hot Reload

Para tener una experiencia de desarrollo ergonómica similar a Laravel, puedes configurar la recarga en caliente (Hot Reload) tanto para el servidor de Rust como para los estilos de Tailwind CSS v4.

### Paso 1: Instalar `cargo-watch` (solo una vez)
Instala la herramienta del ecosistema de Rust:
```bash
cargo install cargo-watch
```

### Paso 2: Ejecutar el entorno con un único comando 🚀
Hemos unificado la compilación continua de Tailwind CSS y la recarga del servidor Rust utilizando un único script en tu archivo `package.json` mediante la librería `concurrently`. 

Para iniciar todo tu entorno de desarrollo al mismo tiempo en una sola terminal, ejecuta:
```bash
npm run dev
```

Este comando levantará en paralelo:
1. El compilador en tiempo real de Tailwind CSS (`npx tailwindcss --watch`).
2. El servidor web Axum con reinicio automático al guardar cambios en archivos `.rs` o `.html` (`cargo watch`).

---

## 📦 Compilación a Producción

Una de las grandes ventajas de Rust es su modelo de distribución. Puedes compilar toda tu aplicación, incluyendo las vistas Askama pre-compiladas, en un único binario altamente optimizado:

```bash
cargo build --release
```

Esto generará un archivo ejecutable independiente en `target/release/larust` (o `larust.exe` en Windows).
Este binario único no requiere dependencias del sistema externo, lo que facilita enormemente el despliegue en entornos en la nube o contenedores Docker.
