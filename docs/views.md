# Vistas e HTMX en Larust 🎨

Larust combina **Askama** (un motor de plantillas HTML compilado de tipo seguro en Rust, muy similar a Blade de Laravel) con **HTMX** para crear aplicaciones web dinámicas y reactivas sin la sobrecarga de un SPA en Javascript.

---

## 🏗️ Creando una Vista con Askama

Las vistas se definen en la carpeta `templates/` con la extensión `.html`.

### 1. Plantilla Base (`templates/layouts/base.html`)
Define la estructura HTML5 común a todas tus páginas utilizando bloques (`block`):

```html
<!DOCTYPE html>
<html lang="es">
<head>
    <meta charset="UTF-8">
    <title>{% block title %}Mi Aplicación{% endblock %}</title>
    <link rel="stylesheet" href="/static/css/output.css">
    <script src="/static/js/htmx.min.js" defer></script>
</head>
<body class="bg-slate-900 text-white">
    <main class="container mx-auto p-6">
        {% block content %}{% endblock %}
    </main>
</body>
</html>
```

### 2. Extender una Plantilla (`templates/home/index.html`)
Para heredar el diseño base y rellenar los bloques:

```html
{% extends "layouts/base.html" %}

{% block title %}Inicio - Larust{% endblock %}

{% block content %}
<h1 class="text-3xl font-bold text-orange-500">Bienvenido a la página</h1>
<p>Este contenido reemplaza al bloque 'content' en base.html.</p>
{% endblock %}
```

---

## ⚡ Reactividad Dinámica con HTMX

HTMX te permite realizar peticiones AJAX directamente desde etiquetas HTML comunes (como `button`, `div`, `form`) y reemplazar parte del DOM con la respuesta HTML devuelta por el servidor.

### Ejemplo de uso de HTMX:

#### En la Vista HTML (`templates/home/index.html`):
```html
<div class="space-y-4">
    <!-- El botón hace una petición GET a /api/ping al hacer click y pone el resultado en #resultado-ping -->
    <button hx-get="/api/ping" 
            hx-target="#resultado-ping" 
            class="px-4 py-2 bg-orange-600 hover:bg-orange-500 rounded">
        Enviar Ping
    </button>

    <div id="resultado-ping" class="text-emerald-400 font-mono">
        <!-- El contenido retornado del servidor se inyectará aquí -->
    </div>
</div>
```

#### En el Controlador (`src/controllers/home_controller.rs`):
```rust
pub async fn ping() -> &'static str {
    "¡Conexión HTMX exitosa! 🚀"
}
```

---

## 🎨 Estilos con Tailwind CSS

Larust utiliza Tailwind CSS v4. Cuando agregues clases de Tailwind en tus archivos `.html` dentro de `templates/`, asegúrate de compilar los estilos para que se reflejen en el navegador.

Ejecuta el compilador en segundo plano durante el desarrollo:
```bash
npx tailwindcss -i ./static/css/input.css -o ./static/css/output.css --watch
```
