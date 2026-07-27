//! Generador de código de Larust (`cargo make:model`, `cargo make:controller`,
//! `cargo make:migration`). Ver la sección "CLI de scaffolding" en CLAUDE.md.
//!
//! No es un binario que se instale aparte: se ejecuta vía los alias de Cargo
//! definidos en `.cargo/config.toml`, que ya vienen en el repo.

use std::fs;
use std::path::Path;

const MODEL_TEMPLATE: &str = r#"use serde::Serialize;
use sqlx::FromRow;

use crate::db::Model;

#[derive(Debug, Clone, Serialize, FromRow)]
pub struct __Pascal__ {
    pub id: String,
    // TODO: agrega aquí las demás columnas de tu tabla `__plural__`
    // (deben coincidir en nombre y orden con tu migración; ver `cargo make:migration`).
}

impl Model for __Pascal__ {
    const TABLE: &'static str = "__plural__";
}

// Si necesitas crear/actualizar filas de `__plural__` desde un formulario,
// además de `Model` implementa `Insertable` (ver src/models/upload.rs para
// un ejemplo) para tener `__Pascal__::create`/`update` genéricos.
"#;

const CONTROLLER_TEMPLATE: &str = r#"use askama::Template;
use axum::extract::State;

use crate::db::{AppState, Model};
use crate::error::AppError;
use crate::models::__Pascal__;

#[derive(Template)]
#[template(path = "__plural__/index.html")]
pub struct __Pascal__sTemplate {
    pub items: Vec<__Pascal__>,
}

pub async fn index(State(state): State<AppState>) -> Result<__Pascal__sTemplate, AppError> {
    let items = __Pascal__::all(&state).await?;

    Ok(__Pascal__sTemplate { items })
}
"#;

const VIEW_TEMPLATE: &str = r#"{% extends "layouts/base.html" %}

{% block title %}__Pascal__s - Larust{% endblock %}

{% block content %}
<div class="max-w-3xl mx-auto space-y-6 py-6">
    <h2 class="text-3xl font-extrabold text-white">__Pascal__s</h2>

    <div class="bg-slate-800 border border-slate-700 rounded-xl overflow-hidden shadow-xl">
        <table class="w-full text-left border-collapse">
            <thead>
                <tr class="border-b border-slate-700 bg-slate-900/50">
                    <th class="p-4 font-semibold text-slate-300">ID</th>
                </tr>
            </thead>
            <tbody class="divide-y divide-slate-700/50 text-slate-300">
                {% if items.is_empty() %}
                <tr>
                    <td class="p-8 text-center text-slate-500">No hay registros todavía.</td>
                </tr>
                {% else %}
                    {% for item in items %}
                    <tr class="hover:bg-slate-700/20 transition">
                        <td class="p-4 font-mono text-sm">{{ item.id }}</td>
                    </tr>
                    {% endfor %}
                {% endif %}
            </tbody>
        </table>
    </div>
</div>
{% endblock %}
"#;

const MIGRATION_TEMPLATE: &str = r#"-- TODO: describe qué hace esta migración.
-- El id es UUIDv7 (texto) generado por la app, no autoincremental: ver la
-- sección "Database / migrations" en CLAUDE.md antes de agregar columnas.
CREATE TABLE IF NOT EXISTS __table__ (
    id VARCHAR(36) PRIMARY KEY
    -- , columna_ejemplo VARCHAR(255) NOT NULL
);
"#;

fn main() {
    let mut args = std::env::args().skip(1);
    let command = args.next();
    let arg = args.next();

    match (command.as_deref(), arg) {
        (Some("make:model"), Some(name)) => make_model(&name),
        (Some("make:controller"), Some(name)) => make_controller(&name),
        (Some("make:migration"), Some(description)) => make_migration(&description),
        _ => print_help(),
    }
}

fn print_help() {
    eprintln!("Uso:");
    eprintln!("  cargo make:model <Nombre>            Ej: cargo make:model Product");
    eprintln!("  cargo make:controller <Nombre>       Ej: cargo make:controller Product");
    eprintln!("  cargo make:migration <descripcion>   Ej: cargo make:migration create_products_table");
    std::process::exit(1);
}

fn make_model(name: &str) {
    let pascal = validate_and_pascalize(name);
    let snake = to_snake_case(&pascal);
    let plural = format!("{}s", snake);
    let path = format!("src/models/{}.rs", snake);

    fail_if_exists(&path);

    let content = render(MODEL_TEMPLATE, &[("__Pascal__", &pascal), ("__plural__", &plural)]);
    fs::write(&path, content).expect("No se pudo escribir el archivo del modelo");
    register_in_mod("src/models/mod.rs", &snake, Some(&pascal));

    println!("✅ Modelo creado: {}", path);
    println!("   Registrado en src/models/mod.rs");
    println!();
    println!("Siguiente paso, crear la migración de la tabla `{}`:", plural);
    println!("   cargo make:migration create_{}_table", plural);
}

fn make_controller(name: &str) {
    let pascal = validate_and_pascalize(name);
    let snake = to_snake_case(&pascal);
    let plural = format!("{}s", snake);
    let controller_path = format!("src/controllers/{}_controller.rs", snake);
    let template_dir = format!("templates/{}", plural);
    let template_path = format!("{}/index.html", template_dir);

    fail_if_exists(&controller_path);
    fail_if_exists(&template_path);

    let controller_content = render(
        CONTROLLER_TEMPLATE,
        &[("__Pascal__", &pascal), ("__plural__", &plural), ("__snake__", &snake)],
    );
    let view_content = render(VIEW_TEMPLATE, &[("__Pascal__", &pascal)]);

    fs::create_dir_all(&template_dir).expect("No se pudo crear el directorio de templates");
    fs::write(&controller_path, controller_content).expect("No se pudo escribir el controlador");
    fs::write(&template_path, view_content).expect("No se pudo escribir la vista");
    register_in_mod("src/controllers/mod.rs", &format!("{}_controller", snake), None);

    println!("✅ Controlador creado: {}", controller_path);
    println!("✅ Vista creada: {}", template_path);
    println!("   Registrado en src/controllers/mod.rs");
    println!();
    println!("Falta agregar la ruta en src/routes.rs, por ejemplo:");
    println!(
        "   .route(\"/{}\", axum::routing::get({}_controller::index))",
        plural, snake
    );
}

fn make_migration(description: &str) {
    validate_migration_description(description);
    let ts = timestamp();
    let filename = format!("migrations/{}_{}.sql", ts, description);

    fail_if_exists(&filename);

    let table_name = description
        .strip_prefix("create_")
        .and_then(|s| s.strip_suffix("_table"))
        .unwrap_or("nombre_de_tabla");

    let content = render(MIGRATION_TEMPLATE, &[("__table__", table_name)]);
    fs::write(&filename, content).expect("No se pudo escribir la migración");

    println!("✅ Migración creada: {}", filename);
}

fn fail_if_exists(path: &str) {
    if Path::new(path).exists() {
        eprintln!("❌ Ya existe {} — bórralo primero si quieres regenerarlo.", path);
        std::process::exit(1);
    }
}

fn validate_and_pascalize(name: &str) -> String {
    let valid = name.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        && name.chars().all(|c| c.is_ascii_alphanumeric());
    if !valid {
        eprintln!(
            "❌ Nombre inválido: \"{}\". Usa solo letras/números, empezando con una letra (ej: Product, ProductCategory).",
            name
        );
        std::process::exit(1);
    }
    to_pascal_case(name)
}

fn validate_migration_description(description: &str) {
    let valid = !description.is_empty()
        && description.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if !valid {
        eprintln!(
            "❌ Descripción inválida: \"{}\". Usa solo letras, números y guiones bajos (ej: create_products_table).",
            description
        );
        std::process::exit(1);
    }
}

fn to_pascal_case(input: &str) -> String {
    let mut chars = input.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn to_snake_case(input: &str) -> String {
    let mut out = String::new();
    for (i, c) in input.chars().enumerate() {
        if c.is_uppercase() {
            if i != 0 {
                out.push('_');
            }
            out.extend(c.to_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

fn render(template: &str, vars: &[(&str, &str)]) -> String {
    let mut out = template.to_string();
    for (key, value) in vars {
        out = out.replace(key, value);
    }
    out
}

fn register_in_mod(mod_path: &str, mod_name: &str, use_type: Option<&str>) {
    let existing = fs::read_to_string(mod_path).unwrap_or_default();
    let mod_decl = format!("pub mod {};", mod_name);
    if existing.contains(&mod_decl) {
        return;
    }

    let mut updated = existing;
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(&mod_decl);
    updated.push('\n');
    if let Some(ty) = use_type {
        updated.push_str(&format!("pub use {}::{};\n", mod_name, ty));
    }

    fs::write(mod_path, updated).expect("No se pudo actualizar el archivo mod.rs");
}

fn timestamp() -> String {
    let now = time::OffsetDateTime::now_utc();
    format!(
        "{:04}{:02}{:02}{:02}{:02}{:02}",
        now.year(),
        u8::from(now.month()),
        now.day(),
        now.hour(),
        now.minute(),
        now.second()
    )
}
