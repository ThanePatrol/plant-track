use axum::{
    extract::Form,
    response::Html,
    routing::{get, post},
    Router,
};

use leptos::*;
use serde::Deserialize;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Row, Sqlite};
use std::{fs, net::SocketAddr};

#[tokio::main]
async fn main() {
    mount_to_body(|cx| view! {cx, <p>"Hello world"</p>})

    /* let app = Router::new()
        .route("/", get(get_home_page))
        .route("/rec", post(print_form))
        .route("/get-main-view", get(get_main_view))
        .route("/get-insert-view", get(get_insert_view));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap(); */
}

async fn print_form(Form(input): Form<Input>) {
    println!("here");
    println!("{:?}", input);
}

#[derive(Deserialize, Debug)]
struct Input {
    name: String,
    email: String,
}

async fn get_home_page() -> Html<String> {
    let file = fs::read_to_string("./resources/index.html").unwrap();
    Html(file)
}

async fn get_insert_view() -> Html<&'static str> {
    Html(
        r##"
	<button hx-get="/get-main-view" 
		hx-trigger="click"
		hx-target="#main-content"
		hx-swap="innerHTML"
		>Show Main View</button>
		<p>Insert View</p>
        "##,
    )
}

async fn get_main_view() -> Html<&'static str> {
    Html(
        r##"
	<button hx-get="/get-insert-view" 
		hx-trigger="click"
		hx-target="#main-content"
		hx-swap="innerHTML"
		>Show Insert View</button>
		<p>Main View</p>
        "##,
    )
}
