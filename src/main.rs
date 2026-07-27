use std::net::SocketAddr;

use larust::{
    build_app,
    db::{new_id, AppState},
    models::User,
};
use sqlx::AnyPool;

#[tokio::main]
async fn main() {
    // Cargar variables de entorno desde el archivo .env
    dotenvy::dotenv().ok();

    let host = std::env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .expect("PORT debe ser un número válido");

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL debe estar configurada en el archivo .env");
    let is_postgres = !database_url.starts_with("sqlite");

    // Instalar los drivers por defecto de sqlx::any (permite alternar dinámicamente SQLite/Postgres)
    sqlx::any::install_default_drivers();

    // Auto-crear archivo de base de datos si es SQLite y no existe
    if database_url.starts_with("sqlite://") {
        let path = database_url.trim_start_matches("sqlite://");
        let path = path.split('?').next().unwrap_or(path);
        if !std::path::Path::new(path).exists() {
            println!("🦀 Creando base de datos SQLite en: {}", path);
            if let Some(parent) = std::path::Path::new(path).parent() {
                std::fs::create_dir_all(parent).ok();
            }
            std::fs::File::create(path).expect("No se pudo crear el archivo de base de datos SQLite");
        }
    }

    // Directorio donde se guardan los archivos subidos por los usuarios
    std::fs::create_dir_all("static/uploads").ok();

    // Inicializar pool de conexiones agnóstico
    let pool = AnyPool::connect(&database_url)
        .await
        .expect("No se pudo conectar a la base de datos");
    let state = AppState::new(pool, is_postgres);

    // Ejecutar migraciones automáticamente de la carpeta /migrations. Todas
    // las tablas usan claves primarias UUID (texto) generadas en la
    // aplicación en vez de autoincrementales, así que el mismo set de
    // migraciones es válido para SQLite y Postgres sin cambios.
    println!("🦀 Ejecutando migraciones de base de datos...");
    sqlx::migrate!("./migrations")
        .run(&state.pool)
        .await
        .expect("No se pudieron ejecutar las migraciones de base de datos");

    // Insertar semilla si la tabla está vacía
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    if count.0 == 0 {
        let password_hash = User::hash_password("password").expect("No se pudo generar el hash de la contraseña semilla");
        sqlx::query(&state.sql("INSERT INTO users (id, name, email, password_hash) VALUES (?, ?, ?, ?)"))
            .bind(new_id())
            .bind("Usuario Larust")
            .bind("info@larust.dev")
            .bind(password_hash)
            .execute(&state.pool)
            .await
            .ok();
        println!("🦀 Semilla de usuario de prueba insertada (info@larust.dev / password).");
    }

    let app = build_app(state);

    let addr_str = format!("{}:{}", host, port);
    let addr: SocketAddr = addr_str.parse().expect("Dirección de red no válida");

    println!("🦀 Larust corriendo en http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
