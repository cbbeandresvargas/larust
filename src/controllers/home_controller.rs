use askama::Template;
use askama_axum::IntoResponse;

#[derive(Template)]
#[template(path = "home/index.html")]
pub struct HomeTemplate;

pub async fn index() -> impl IntoResponse {
    HomeTemplate
}

pub async fn ping() -> &'static str {
    "¡Conexión HTMX exitosa desde el servidor Larust! 🚀"
}
