use axum::{
    extract::{Form, State},
    response::Html,
    routing::{get, post},
    Router,
};

use anyhow::Result;

use leptos::ssr::*;
use serde::Deserialize;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{Pool, Postgres,};
use std::fs;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

type AppState = Arc<Mutex<App>>;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::from_path("./.env").expect("Error reading .env");

    let pool = init_pool(&dotenvy::var("DB_URL").unwrap()).await?;

    let app_state = Arc::new(Mutex::new(App {
        db_pool: pool,
        state: Vec::new(),
    }));

    let app = Router::new()
        .route("/", get(index))
        .route("/rec", post(print_form))
        .route("/get-main-view", get(get_main_view))
        .route("/get-insert-view", get(get_insert_view))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
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

#[derive(Clone)]
pub struct App {
    pub db_pool: Pool<Postgres>,
    pub state: Vec<i32>, //todo - some proper state for the app as a whole
}

#[derive(Deserialize, Debug, sqlx::FromRow)]
pub struct Users {
    pub user_id: u32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: Option<String>,
}

#[derive(Deserialize, Debug, sqlx::FromRow)]
pub struct Plant {
    pub plant_id: i32,
    pub user_id: i32,
    pub botanical_name: String,
    pub common_name: String,
    pub last_fed: time::Date,
    pub feed_interval: i32, // days
    pub last_potted: time::Date,
    pub potting_interval: i32,
    pub last_pruned: time::Date,
    pub pruning_interval: i32,
}

#[derive(Deserialize, Debug, sqlx::FromRow)]
pub struct PlantPhoto {
    pub plant_id: i32,
    pub user_id: i32,
    pub photo_uri: String,
}

#[derive(Deserialize, Debug, sqlx::FromRow)]
pub struct Comments {
    pub plant_id: i32, 
    pub user_id: i32,
    pub time_made: time::OffsetDateTime,
    pub comment: String,
}


async fn index(State(app): State<AppState>) -> Html<String> {
    let pool = &app.lock().await.db_pool;
    
    let file = fs::read_to_string("./resources/index.html").unwrap();
    Html(file)
}

async fn get_all_plants(pool: &Pool<Postgres>, user_id: i32) -> Vec<Plant> {
    let all_plants = sqlx::query("SELECT * FROM plants WHERE plants.user_id == ?;")
        .bind(user_id)
        .fetch_all(pool)
        .await
        .unwrap_or(Vec::new())
        .into_iter()
        .map(|row| pl)
}

async fn get_insert_view() -> Html<&'static str> {
    Html(
        r##"
	<button hx-get="/get-main-view" 
		hx-trigger="click"
		hx-target="#main-content"
		hx-swap="innerHTML"
		>Show Main View</button>
		<p>Insert View, something something</p>
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

async fn init_pool(db_url: &str) -> Result<Pool<Postgres>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;
    Ok(pool)
}
