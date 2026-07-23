use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use larust::build_app;
use sqlx::{any::AnyPoolOptions, AnyPool};
use tower::ServiceExt;

async fn test_pool() -> AnyPool {
    sqlx::any::install_default_drivers();
    // `sqlite::memory:` gives each pooled connection its own separate
    // in-memory database, so migrations run on one connection would be
    // invisible to queries on another. Capping the pool at a single
    // connection keeps every query on the same in-memory database.
    let pool = AnyPoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn ping_returns_ok() {
    let app = build_app(test_pool().await);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/ping")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
}

#[tokio::test]
async fn users_page_requires_authentication() {
    let app = build_app(test_pool().await);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "/login");
}

#[tokio::test]
async fn register_then_access_protected_users_page() {
    let app = build_app(test_pool().await);

    let register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("name=Test&email=test%40example.com&password=secret123"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(register_response.status(), StatusCode::SEE_OTHER);
    let cookie = register_response
        .headers()
        .get("set-cookie")
        .expect("se espera una cookie de sesión tras registrarse")
        .to_str()
        .unwrap()
        .to_string();

    let users_response = app
        .oneshot(
            Request::builder()
                .uri("/users")
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(users_response.status(), StatusCode::OK);
}

#[tokio::test]
async fn login_with_wrong_password_is_rejected() {
    let app = build_app(test_pool().await);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/login")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("email=info%40larust.dev&password=wrong"))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get("set-cookie").is_none());
}

#[tokio::test]
async fn upload_file_then_appears_in_list() {
    let app = build_app(test_pool().await);

    let register_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/register")
                .header("content-type", "application/x-www-form-urlencoded")
                .body(Body::from("name=Uploader&email=uploader%40example.com&password=secret123"))
                .unwrap(),
        )
        .await
        .unwrap();
    let cookie = register_response
        .headers()
        .get("set-cookie")
        .unwrap()
        .to_str()
        .unwrap()
        .to_string();

    let boundary = "X-BOUNDARY";
    let body = format!(
        "--{b}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"hello.txt\"\r\nContent-Type: text/plain\r\n\r\nhola\r\n--{b}--\r\n",
        b = boundary
    );

    let upload_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/uploads")
                .header("cookie", cookie.clone())
                .header("content-type", format!("multipart/form-data; boundary={}", boundary))
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(upload_response.status(), StatusCode::SEE_OTHER);

    let list_response = app
        .oneshot(
            Request::builder()
                .uri("/uploads")
                .header("cookie", cookie)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(list_response.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(list_response.into_body(), usize::MAX)
        .await
        .unwrap();
    let html = String::from_utf8_lossy(&bytes);
    assert!(html.contains("hello.txt"));

    // El controlador escribe el archivo en el static/uploads/ real del repo;
    // se limpia aquí para no dejar artefactos de la prueba.
    for entry in std::fs::read_dir("static/uploads").unwrap().flatten() {
        if entry.file_name().to_string_lossy().ends_with("-hello.txt") {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

#[tokio::test]
async fn unknown_route_returns_404() {
    let app = build_app(test_pool().await);

    let response = app
        .oneshot(
            Request::builder()
                .uri("/does-not-exist")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
