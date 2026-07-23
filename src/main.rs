use axum::Router;
use tower_http::services::ServeDir;
use std::net::SocketAddr;
use sqlx::AnyPool;

mod controllers;
mod models;
mod routes;

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

    // Inicializar pool de conexiones agnóstico
    let pool = AnyPool::connect(&database_url)
        .await
        .expect("No se pudo conectar a la base de datos");

    // Ejecutar migraciones automáticamente de la carpeta /migrations
    println!("🦀 Ejecutando migraciones de base de datos...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("No se pudieron ejecutar las migraciones de base de datos");

    // Insertar semilla si la tabla está vacía
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap_or((0,));

    if count.0 == 0 {
        sqlx::query("INSERT INTO users (name, email) VALUES ('Usuario Larust', 'info@larust.dev')")
            .execute(&pool)
            .await
            .ok();
        println!("🦀 Semilla de usuario de prueba insertada con éxito.");
    }

    // Construir la aplicación con rutas web, API y archivos estáticos
    let app = Router::new()
        .merge(routes::web_routes())
        .nest("/api", routes::api_routes())
        .nest_service("/static", ServeDir::new("static"))
        .with_state(pool);

    let addr_str = format!("{}:{}", host, port);
    let addr: SocketAddr = addr_str.parse().expect("Dirección de red no válida");
    
    println!("🦀 Larust corriendo en http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
