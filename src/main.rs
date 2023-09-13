use axum::{
    extract::{Form, State},
    response::Html,
    routing::{get, post},
    Router,
};

use anyhow::Result;

use leptos::view;
use leptos::*;
use serde::Deserialize;
use sqlx::postgres::{PgPoolOptions, PgRow};
use sqlx::{FromRow, Row};
use sqlx::{Pool, Postgres};
use std::fs;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

type AppState = Arc<Mutex<App>>;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::from_path("./.env").expect("Error reading .env");

    let pool = init_pool(&dotenvy::var("DATABASE_URL").unwrap()).await?;

    let app_state = Arc::new(Mutex::new(App {
        db_pool: pool,
        state: Vec::new(),
    }));

    let app = Router::new()
        .route("/", get(index))
        .route("/get-main-view", get(get_main_view))
        .route("/get-insert-view", get(get_insert_view))
        .route("/add-plant", post(post_add_plant))
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
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

#[derive(Deserialize, Debug, Clone)]
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

impl FromRow<'_, PgRow> for Plant {
    fn from_row(row: &PgRow) -> sqlx::Result<Self> {
        Ok(Self {
            plant_id: row.try_get("plant_id")?,
            user_id: row.try_get("user_id")?,
            botanical_name: row.try_get("botanical_name")?,
            common_name: row.try_get("common_name")?,
            last_fed: row.try_get("last_fed")?,
            feed_interval: row.try_get("feed_interval")?,
            last_potted: row.try_get("last_potted")?,
            potting_interval: row.try_get("potting_interval")?,
            last_pruned: row.try_get("last_pruned")?,
            pruning_interval: row.try_get("pruning_interval")?,
        })
    }
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
    let plants = get_all_plants(pool, 1).await.unwrap();
    let file = fs::read_to_string("./resources/index.html").unwrap();

    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx,
            <head>
                <script src="https://unpkg.com/htmx.org@1.9.2" integrity="sha384-L6OqL9pRWyyFU3+/bjdSri+iIphTN/bvYyM37tICVyOJkWZLpP2vGn6VUEXgzg6h" crossorigin="anonymous"></script>
            </head>
            <body>
                <AddPlantView
                    user_id=1
                />
            </body>

        }
    });

    Html(html)
}

async fn get_all_plants(pool: &Pool<Postgres>, user_id: i32) -> Result<Vec<Plant>> {
    let rows = sqlx::query_as(r#"SELECT * FROM plants WHERE user_id = 1"#)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

async fn post_add_plant(State(app): State<AppState>, Form(plant): Form<Plant>) -> Html<String> {
    println!("here");
    println!("{:?}", plant);

    let html = leptos::ssr::render_to_string(move |cx| {
        view! {cx,
            <PlantAddSuccess/>
        }
    });
    Html(html)
}

#[component]
pub fn PlantAddSuccess(cx: Scope) -> impl IntoView {
    view! {cx,
        <p>"Plant added successfully"</p>
    }
}

#[component]
pub fn AddPlantView(cx: Scope, user_id: i32) -> impl IntoView {
    view! { cx,
        <div id="add-view">
            <form>
                <input type="hidden" name="plant_id" value=10/>
                <input type="hidden" name="user_id" value=user_id.to_string()/>
                <label for="botanical_name">Botanical name: </label>
                <input type="text" name="botanical_name" id="botanical_name" required />
                <label for="common_name">Common name: </label>
                <input type="text" name="common_name" id="common_name" required />
                <label for="last_fed">Last fertilized: </label>
                <input type="date" name="last_fed" id="last_fed" required />
                <label for="feed_interval">Fertilizing interval in days: </label>
                <input type="number" name="feed_interval" id="feed_interval" required />
                <label for="last_potted">Last Potted: </label>
                <input type="date" name="last_potted" id="last_potted" required />
                <label for="potting_interval">Potting interval in days: </label>
                <input type="number" name="potting_interval" id="potting_interval" required />
                <label for="last_pruned">Last pruned: </label>
                <input type="date" name="last_pruned" id="last_pruned" required />
                <label for="pruning_interval">Pruning interval in days: </label>
                <input type="number" name="pruning_interval" id="pruning_interval" required />
                <input type="submit"
                    hx-post="/add-plant"
                    hx-trigger="click"
                    hx-target="#add-view"
                    hx-swap="outerHTML"
                    >Add new plant</input>

            </form>
        </div>

    }
}

#[component]
pub fn PlantView(cx: Scope, plants: Vec<Plant>) -> impl IntoView {
    let (plants, _) = create_signal(cx, plants);

    view! { cx,
        <ul id="plants">
            <For
                //get each item we iterate over
                each=move || plants.get()
                key=|plant| plant.plant_id
                view=move |cx, plant: Plant| {
                    view! { cx,
                        <PlantItem plant=plant />
                    }
                }
            />
        </ul>
    }
}

#[component]
pub fn PlantItem(cx: Scope, plant: Plant) -> impl IntoView {
    view! { cx,
        <div>
            <div>botanical name: {plant.botanical_name}</div>
        </div>
    }
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
