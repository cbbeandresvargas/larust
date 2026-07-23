use axum::extract::State;
use askama::Template;
use askama_axum::IntoResponse;
use sqlx::AnyPool;
use crate::models::User;

#[derive(Template)]
#[template(path = "users/index.html")]
pub struct UsersTemplate {
    pub users: Vec<User>,
}

pub async fn index(State(db): State<AnyPool>) -> impl IntoResponse {
    let users = sqlx::query_as::<_, User>("SELECT id, name, email FROM users")
        .fetch_all(&db)
        .await
        .unwrap_or_default();

    UsersTemplate { users }
}
