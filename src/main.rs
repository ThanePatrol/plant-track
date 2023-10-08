use axum::{
    async_trait, debug_handler,
    extract::{Form, FromRequestParts, State},
    http::{request::Parts, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Response},
    routing::{get, get_service, post},
    Extension, Json, RequestPartsExt, Router,
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use anyhow::Result;

use leptos::view;
use leptos::*;
use mail_send::mail_builder::MessageBuilder;
use serde::Deserialize;
use serde_json::json;
use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};
use sqlx::{Pool, Postgres};
use std::rc::Rc;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;

use tower_http::services::ServeDir;

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::Lazy;
use tokio::io::{AsyncRead, AsyncWrite};
mod components;
use components::*;

mod db_api;
use db_api::*;

mod auth_memes;
use crate::auth_memes::{authorize, check_client, AuthError};

type AppState = Arc<Mutex<App>>;

static DOES_NOT_NEED: i32 = 100_000; //Sentinel value for a fertilizer/pot/prune interval where
                                     //the user marks it as not relevant for a particular plant
static N_PLANTS: i32 = 9;

static KEYS: Lazy<Keys> = Lazy::new(|| {
    dotenvy::from_path("./.env").expect("Error reading .env");
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

#[tokio::main]
async fn main() -> Result<()> {
    let pool = init_pool(&dotenvy::var("DATABASE_URL").unwrap()).await?;

    let app_state = Arc::new(Mutex::new(App {
        db_pool: pool,
        //state: Vec::new(),
    }));

    let app = Router::new()
        .nest_service("/css", get_service(ServeDir::new("resources/css")))
        .route("/", get(index))
        .route("/add-plant", post(post_add_plant))
        .route("/plant-view", get(get_plant_view))
        .route("/add-view", get(get_add_view))
        .route("/sort-by-feed", get(get_sorted_feed_plant_view))
        .route("/update-view", post(get_update_view))
        .route("/update-plant", post(post_update_plant))
        .route("/search-plants", post(search_plants))
        .route("/get-plants-that-need-attention", get(get_plants_attn))
        .route("/login", get(get_login_page))
        .route("/auth", post(authorize))
        .layer(middleware::from_fn(check_client))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await?;

    Ok(())
}

pub struct Keys {
    pub encoding: EncodingKey,
    pub decoding: DecodingKey,
}

impl Keys {
    fn new(secret: &[u8]) -> Self {
        Self {
            encoding: EncodingKey::from_secret(secret),
            decoding: DecodingKey::from_secret(secret),
        }
    }
}

#[derive(Clone)]
pub struct App {
    pub db_pool: Pool<Postgres>,
    //pub state: Vec<i32>, //todo - some proper state for the app as a whole
}

#[derive(Debug)]
pub struct UserState {
    user_id: usize,
    plants: Rc<Vec<Plant>>, //to cache the current list of plants, rather than always hitting db
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

#[derive(Deserialize, Debug, Clone)]
pub struct PlantID {
    pub plant_id: i32,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Search {
    pub search_string: String,
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

#[debug_handler]
async fn index(State(app): State<AppState>, user_id: Option<Extension<i32>>) -> Html<String> {
    println!("user_id {:?}", user_id);
    let pool = &app.lock().await.db_pool;
    let plants = get_all_plants(pool, 1, N_PLANTS.to_string()).await.unwrap();

    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx,
            <head>
                <script src="https://unpkg.com/htmx.org@1.9.2" integrity="sha384-L6OqL9pRWyyFU3+/bjdSri+iIphTN/bvYyM37tICVyOJkWZLpP2vGn6VUEXgzg6h" crossorigin="anonymous"></script>
                <script src="https://unpkg.com/htmx.org/dist/ext/json-enc.js"></script>
                <link href="https://fonts.googleapis.com/css?family=Roboto:100,300,400,500,700,900" rel="stylesheet"/>
                <link rel="stylesheet" href="./css/styles.css"/>
            </head>
            <body>
                <LoginView/>
                <MainView
                    plants=plants
                />
            </body>
        }
    });

    Html(html)
}

async fn get_login_page(State(app): State<AppState>) -> Html<String> {
    let html = leptos::ssr::render_to_string(move |cx| {
        view! {cx,
            <LoginView/>
        }
    });

    Html(html)
}

pub async fn get_add_view(State(_app): State<AppState>) -> Html<String> {
    let html = leptos::ssr::render_to_string(move |cx| {
        view! {cx,
            <AddPlantView
                user_id=1 //todo - state based value
                plant_id=None
                text="Add Plant".into()
            />
        }
    });
    Html(html)
}

pub async fn get_plant_view(State(app): State<AppState>) -> Html<String> {
    let pool = &app.lock().await.db_pool;
    let plants = get_all_plants(pool, 1, N_PLANTS.to_string()).await.unwrap(); //todo - show default screen

    let html = leptos::ssr::render_to_string(move |cx| {
        view! {cx,
            <PlantView
                plants=plants
            />
        }
    });
    Html(html)
}

pub async fn get_sorted_feed_plant_view(State(app): State<AppState>) -> Html<String> {
    let pool = &app.lock().await.db_pool;
    let mut plants = get_all_plants(pool, 1, N_PLANTS.to_string()).await.unwrap();
    plants.sort_by(|a, b| a.last_fed.cmp(&b.last_fed));
    let html = leptos::ssr::render_to_string(move |cx| {
        view! {cx,
            <PlantView
                plants=plants
            />
        }
    });
    Html(html)
}

#[debug_handler]
pub async fn post_add_plant(
    State(app): State<AppState>,
    Form(mut plant): Form<Plant>,
) -> Html<String> {
    let pool = &app.lock().await.db_pool;

    let html;

    let plant_id = db_api::add_plant_to_db(pool, plant.clone()).await;

    match plant_id {
        Ok(id) => {
            plant.plant_id = id;
            html = leptos::ssr::render_to_string(move |cx| {
                view! {cx,
                    <PlantAddSuccess
                        plant=plant
                    />
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
                       plant_id=None
                       text="Add Plant".into()
                    />
                }
            });
        }
    }

    Html(html)
}

pub async fn get_update_view(
    State(app): State<AppState>,
    Form(plant_id): Form<PlantID>,
) -> Html<String> {
    let pool = &app.lock().await.db_pool;
    let plant = db_api::get_plant_from_id(pool, 1, plant_id.plant_id)
        .await
        .unwrap();

    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx,
            <UpdateView
                plant=plant
                user_id=1
            />

        }
    });

    Html(html)
}

pub async fn post_update_plant(
    State(app): State<AppState>,
    Form(plant): Form<Plant>,
) -> Html<String> {
    let pool = &app.lock().await.db_pool;
    db_api::update_plant(pool, plant).await.unwrap();
    let plants = get_all_plants(pool, 1, N_PLANTS.to_string()).await.unwrap();

    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx,
            <PlantView
                plants=plants
            />
        }
    });
    Html(html)
}

pub async fn search_plants(
    State(app): State<AppState>,
    Form(search): Form<Search>,
) -> Html<String> {
    let pool = &app.lock().await.db_pool;

    let plants = db_api::search_plants(pool, search.search_string, 1)
        .await
        .unwrap(); //TODO - proper user id

    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx,
            <PlantView
                plants=plants
            />
        }
    });
    Html(html)
}

pub async fn get_plants_attn(State(app): State<AppState>) -> Html<String> {
    let pool = &app.lock().await.db_pool;
    let plants = db_api::get_plants_that_need_attention(pool, 1)
        .await
        .expect("Error getting plants");
    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx,
            <PlantView
                plants=plants
            />
        }
    });
    Html(html)
}

pub async fn notify_users_of_required_actions(State(app): State<AppState>) {
    let pool = &app.lock().await.db_pool;

    //TODO - loop through users instead of hardcoding a single user

    let plants = db_api::get_plants_that_need_attention(pool, 1)
        .await
        .expect("Error getting plants");
    if plants.is_empty() {
        return; //TODO - change to continue in a loop
    }

    let email = db_api::get_user_email(pool, 1)
        .await
        .expect("Couldn't find user email!");

    let mut client = mail_send::SmtpClientBuilder::new("smtp-mail.outlook.com", 587)
        .implicit_tls(false)
        .credentials((
            "Thane_Patrol@outlook.com",
            std::env::var("EMAIL_PASSWORD").unwrap().as_str(),
        ))
        .connect()
        .await
        .expect("Error connecting to client");

    send_email(email, plants, &mut client).await;
}

fn get_days_till_next_feed(plant: &Plant) -> (String, String, String) {
    fn do_sub(days: i32) -> i64 {
        let cur_date = time::OffsetDateTime::now_utc();

        let next_date = cur_date + time::Duration::days(days as i64);

        (next_date - cur_date).whole_days()
    }
    let feed_int = do_sub(plant.feed_interval);
    let pot_int = do_sub(plant.potting_interval);
    let prune_int = do_sub(plant.pruning_interval);
    let sentinel = String::from("Overdue");

    (
        match feed_int.cmp(&0) {
            std::cmp::Ordering::Greater => feed_int.to_string(),
            _ => sentinel.clone(),
        },
        match pot_int.cmp(&0) {
            std::cmp::Ordering::Greater => pot_int.to_string(),
            _ => sentinel.clone(),
        },
        match prune_int.cmp(&0) {
            std::cmp::Ordering::Greater => prune_int.to_string(),
            _ => sentinel.clone(),
        },
    )
}

async fn send_email<T: AsyncRead + AsyncWrite + Unpin>(
    email: String,
    plants: Vec<Plant>,
    mail: &mut mail_send::SmtpClient<T>,
) {
    let html = leptos::ssr::render_to_string(move |cx| {
        view! {cx,
            <EmailItem
                plants=plants
            />
        }
    });
    let message = MessageBuilder::new()
        .from(("Thane Patrol", "thane_patrol@outlook.com"))
        .to(vec![("1", email.as_str())])
        .subject("Plants in need of your care!")
        .html_body(html);

    mail.send(message)
        .await
        .expect(format!("Error sending email to: {}", email).as_str());
}
