use crate::{Cookie, IntoResponse, PgPool, Uri};
use axum::extract::{Extension, Form, Path};
use axum::response::{Html, Redirect};
use serde::{Deserialize, Serialize};
use tera::{Context, Tera};
use tower_cookies::Cookies;

#[derive(Serialize)]
pub struct Todo {
    id: i32,
    title: String,
    description: String,
    checked: bool,
    login: String,
}

pub async fn list_todos(
    Extension(pool): Extension<PgPool>,
    Extension(tera): Extension<Tera>,
    cookies: Cookies,
) -> Html<String> {
    match cookies.get("login") {
        Some(login) => {
            let is_doesnt_exist = sqlx::query!(
                "SELECT COUNT(1) FROM accounts WHERE login = $1",
                login.value().to_string()
            )
            .fetch_one(&pool)
            .await
            .unwrap();

            if login.value().to_string().is_empty() || is_doesnt_exist.count.unwrap() == 0 {
                return Html(tera.render("login.html", &Context::new()).unwrap());
            }
        }
        None => {
            cookies.add(Cookie::new("login", String::new()));
            cookies.add(Cookie::new("password", String::new()));

            return Html(tera.render("login.html", &Context::new()).unwrap());
        }
    }

    let username = cookies.get("login").unwrap().value().to_string();
    let todos = sqlx::query_as!(Todo, "SELECT * FROM todos WHERE login = $1", username)
        .fetch_all(&pool)
        .await
        .unwrap();

    let mut context = Context::new();
    context.insert("todos", &todos);
    context.insert("account", &username);

    Html(tera.render("index.html", &context).unwrap())
}

pub async fn get_description(
    Path(id): Path<u32>,
    Extension(pool): Extension<PgPool>,
    Extension(tera): Extension<Tera>,
) -> Html<String> {
    let todo = sqlx::query!("SELECT * FROM todos WHERE id = $1", id as i32)
        .fetch_one(&pool)
        .await
        .unwrap();

    let mut context = Context::new();
    context.insert("title", &todo.title);
    context.insert("description", &todo.description);
    context.insert("id", &todo.id);
    context.insert(
        "checked",
        &if todo.checked { "Done" } else { "Not yet Done" },
    );

    Html(tera.render("description.html", &context).unwrap())
}

pub async fn delete_all_done_todos(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    sqlx::query!(
        "DELETE FROM todos WHERE checked = true AND login = $1",
        cookies.get("login").unwrap().value().to_string()
    )
    .execute(&pool)
    .await
    .unwrap();

    Redirect::to(Uri::from_static("/"))
}

pub async fn delete_all_todos(
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    sqlx::query!(
        "DELETE FROM todos WHERE login = $1",
        cookies.get("login").unwrap().value().to_string()
    )
    .execute(&pool)
    .await
    .unwrap();

    Redirect::to(Uri::from_static("/"))
}

pub async fn delete_todo(
    Path(id): Path<u32>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    sqlx::query!("DELETE FROM todos WHERE id = $1", id as i32)
        .execute(&pool)
        .await
        .unwrap();

    Redirect::to(Uri::from_static("/"))
}

#[derive(Deserialize)]
pub struct NewTodo {
    title: String,
    description: String,
}

pub async fn editing_new_todo<'a>() -> Html<&'a str> {
    Html(include_str!("../templates/new.html"))
}

pub async fn create_todo(
    Form(todo): Form<NewTodo>,
    cookies: Cookies,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    sqlx::query!(
        "INSERT INTO todos(title, description, login) VALUES($1, $2, $3)",
        todo.title,
        todo.description,
        cookies.get("login").unwrap().value().to_string()
    )
    .execute(&pool)
    .await
    .unwrap();

    Redirect::to(Uri::from_static("/"))
}

#[derive(Deserialize)]
pub struct UpdatedTodo {
    title: String,
    description: String,
    checked: Option<String>,
}

pub async fn edit_todo(
    Path(id): Path<u32>,
    Extension(pool): Extension<PgPool>,
    Extension(tera): Extension<Tera>,
) -> Html<String> {
    let todo = sqlx::query_as!(Todo, "SELECT * FROM todos WHERE id = $1", id as i32)
        .fetch_one(&pool)
        .await
        .unwrap();

    let mut context = Context::new();
    context.insert("todo", &todo);

    Html(tera.render("edit.html", &context).unwrap())
}

pub async fn update_todo(
    Path(id): Path<u32>,
    Form(new_content): Form<UpdatedTodo>,
    Extension(pool): Extension<PgPool>,
) -> impl IntoResponse {
    sqlx::query!(
        "UPDATE todos SET title = $1, description = $2, checked = $3 WHERE id = $4",
        new_content.title,
        new_content.description,
        new_content.checked.is_some(),
        id as i32,
    )
    .execute(&pool)
    .await
    .unwrap();

    Redirect::to(Uri::from_static("/"))
}
