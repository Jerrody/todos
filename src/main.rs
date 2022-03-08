mod login;
mod todos;

use login::*;
use todos::*;

use axum::{
    extract::Extension,
    http::uri::Uri,
    response::IntoResponse,
    routing::{get, post},
};
use sqlx::PgPool;
use tera::Tera;
use tower_cookies::{Cookie, CookieManagerLayer};

#[tokio::main]
async fn main() {
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:1597@localhost:3000/postgres".to_string());
    let pool = PgPool::connect(&db_url).await.unwrap();
    let tera = Tera::new("templates/*.html").unwrap();

    let app = axum::Router::new()
        .route("/", get(list_todos))
        .route("/login", get(login_into_account))
        .route("/register", get(register_page).post(register))
        .route("/logout", get(logout))
        .route("/new", get(editing_new_todo).post(create_todo))
        .route("/edit/:id", get(edit_todo).post(update_todo))
        .route("/:id", post(delete_todo).get(get_description))
        .route("/reset", get(delete_all_todos).post(delete_all_done_todos))
        .layer(Extension(pool))
        .layer(CookieManagerLayer::new())
        .layer(Extension(tera));

    let address = std::net::SocketAddr::from(([127, 0, 0, 1], 8000));
    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
