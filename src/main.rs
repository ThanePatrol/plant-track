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
use std::{fmt::Display, fs};
use time::{format_description, Duration};

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
        .route("/add-plant", post(post_add_plant))
        .route("/plant-view", get(get_plant_view))
        .route("/add-view", get(get_add_view))
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
                <MainView
                    plants=plants
                />
            </body>

        }
    });

    Html(html)
}

async fn get_all_plants(pool: &Pool<Postgres>, user_id: i32) -> Result<Vec<Plant>> {
    let rows = sqlx::query_as(r#"SELECT * FROM plants WHERE user_id = $1"#)
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

//todo add validation that checks if user_id is the same as what was submitted in the form
async fn post_add_plant(State(app): State<AppState>, Form(plant): Form<Plant>) -> Html<String> {
    let pool = &app.lock().await.db_pool;

    let html;

    match sqlx::query(
        "INSERT INTO plants (user_id, botanical_name, common_name, last_fed, feed_interval, \
            last_potted, potting_interval, last_pruned, pruning_interval)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ",
    )
    .bind(plant.user_id)
    .bind(plant.botanical_name)
    .bind(plant.common_name)
    .bind(plant.last_fed)
    .bind(plant.feed_interval)
    .bind(plant.last_potted)
    .bind(plant.potting_interval)
    .bind(plant.last_pruned)
    .bind(plant.pruning_interval)
    .execute(pool)
    .await
    {
        Ok(_) => {
            html = leptos::ssr::render_to_string(move |cx| {
                view! {cx,
                    <PlantAddSuccess/>
                }
            });
        }
        Err(e) => {
            html = leptos::ssr::render_to_string(move |cx| {
                view! {cx,
                    <PlantAddFailure
                        error = e.to_string()
                    />
                    <AddPlantView
                       user_id = plant.user_id
                    />
                }
            });
        }
    }

    Html(html)
}

pub async fn get_add_view(State(app): State<AppState>) -> Html<String> {
    let html = leptos::ssr::render_to_string(move |cx| {
        view! {cx,
            <AddPlantView
                user_id=1 //todo - state based value
            />
        }
    });
    Html(html)
}

pub async fn get_plant_view(State(app): State<AppState>) -> Html<String> {
    let pool = &app.lock().await.db_pool;
    let plants = get_all_plants(pool, -1).await.unwrap(); //todo - show default screen

    let html = leptos::ssr::render_to_string(move |cx| {
        view! {cx,
            <PlantView
                plants=plants
            />
        }
    });
    Html(html)
}

#[component]
pub fn MainView(cx: Scope, plants: Vec<Plant>) -> impl IntoView {
    view! {cx,
        <div class="button-bar">
            <button id="view-button"
                hx-get="/plant-view"
                hx-trigger="click"
                hx-target="#main-view"
                hx-swap="innerHTML"
            >"View plants"</button>

            <button id="update-button">"Update plants"</button> //todo - add view plant func.
            <button id="add-button"
                hx-get="/add-view"
                hx-trigger="click"
                hx-target="#main-view"
                hx-swap="innerHTML"
            >"Add plant"</button>
        </div>
        <main id="main-view">
            <PlantView
                plants=plants
            />
        </main>

    }
}

#[component]
pub fn PlantAddSuccess(cx: Scope) -> impl IntoView {
    view! {cx,
        <p>"Plant added successfully"</p>
    }
}

#[component]
pub fn PlantAddFailure(cx: Scope, error: String) -> impl IntoView {
    view! {cx,
        <p>"Plant not added! Please trying adding it again"</p>
        <p>"The following error code was encounted:" {move || error.clone()}</p>
    }
}

/// Form for adding plants, user_id is prefilled on server.
///
#[component]
pub fn AddPlantView(cx: Scope, user_id: i32) -> impl IntoView {
    view! { cx,
        <div id="add-view">
            <form>
                <input type="hidden" name="plant_id" value=-1/>
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
    let (feed_days, pot_days, prune_days) = get_days_till_next_feed(&plant);
    let format = format_description::parse("[year]-[month]-[day]").unwrap();
    let feed_date = (plant.last_fed.format(&format)).unwrap();
    let pot_date = (plant.last_potted.format(&format)).unwrap();
    let prune_date = (plant.last_pruned.format(&format)).unwrap();

    view! { cx,
        <div class="plant-container">
            <div>Botanical name: {plant.botanical_name}</div>
            <div>Common name: {plant.common_name}</div>
            <div>Last fed: {feed_date}</div>
            <div>Time to next feeding cycle: {feed_days}</div>
            <div>Last potted: {pot_date}</div>
            <div>Time to next potting cycle: {pot_days}</div>
            <div>Last fed: {prune_date}</div>
            <div>Time to next pruning cycle: {prune_days}</div>
        </div>
    }
}

async fn init_pool(db_url: &str) -> Result<Pool<Postgres>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;
    Ok(pool)
}

fn get_days_till_next_feed(plant: &Plant) -> (i64, i64, i64) {
    fn do_sub(last: time::Date, days: i32) -> i64 {
        let cur_date = time::OffsetDateTime::now_utc();

        let next_date = cur_date + time::Duration::days(days as i64);

        (next_date - cur_date).whole_days()
    }

    (
        do_sub(plant.last_fed, plant.feed_interval),
        do_sub(plant.last_potted, plant.potting_interval),
        do_sub(plant.last_pruned, plant.pruning_interval),
    )
}
