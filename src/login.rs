use crate::{Cookie, IntoResponse, PgPool, Uri};
use axum::extract::{Extension, Form};
use axum::response::{Html, Redirect};
use serde::Deserialize;
use tera::{Context, Tera};
use tower_cookies::Cookies;

const IS_ALREADY_EXISTS_ACCOUNT: &str = "Is already exists account with this login!";

#[derive(Deserialize)]
pub struct Account {
    login: String,
    password: String,
}

pub async fn login_into_account(
    Form(account): Form<Account>,
    Extension(pool): Extension<PgPool>,
    cookies: Cookies,
) -> impl IntoResponse {
    let is_exists = sqlx::query!(
        "SELECT * FROM accounts WHERE login = $1 AND password = $2",
        account.login,
        account.password
    )
    .fetch_one(&pool)
    .await;

    if let Ok(account) = is_exists {
        cookies.add(Cookie::new("login", account.login));
        cookies.add(Cookie::new("password", account.password));
    }

    Redirect::to(Uri::from_static("/"))
}

pub async fn register_page(Extension(tera): Extension<Tera>) -> Html<String> {
    let mut context = Context::new();
    context.insert("is_already_exists_account", "");

    Html(tera.render("register.html", &context).unwrap())
}

pub async fn register(
    Form(new_account): Form<Account>,
    Extension(pool): Extension<PgPool>,
    Extension(tera): Extension<Tera>,
    cookies: Cookies,
) -> impl IntoResponse {
    let is_already_exists_login = sqlx::query!(
        "SELECT COUNT(1) FROM accounts WHERE login = $1",
        new_account.login
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    match is_already_exists_login.count.unwrap() {
        0 => {
            sqlx::query!(
                "INSERT INTO accounts(login, password) VALUES($1, $2)",
                new_account.login,
                new_account.password
            )
                .execute(&pool)
                .await
                .unwrap();

            cookies.add(Cookie::new("login", new_account.login));
            cookies.add(Cookie::new("password", new_account.password));

            Redirect::to(Uri::from_static("/")).into_response()
        }
        _ => {
            let mut context = Context::new();
            context.insert("is_already_exists_account", IS_ALREADY_EXISTS_ACCOUNT);

            Html(tera.render("register.html", &context).unwrap()).into_response()
        }
    }
}

pub async fn logout(cookies: Cookies) -> impl IntoResponse {
    cookies.add(Cookie::new("login", String::new()));
    cookies.add(Cookie::new("password", String::new()));

    Redirect::to(Uri::from_static("/"))
}
