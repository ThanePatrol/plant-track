use auth_memes::{check_password, get_jwt_cookie_for_new_user, hash_password};
use axum::{
    debug_handler,
    extract::{Form, State},
    http::{
        header::{self},
        StatusCode,
    },
    middleware::{self},
    response::{Html, IntoResponse, Response},
    routing::{get, get_service, post},
    Extension, Router,
};

use anyhow::Result;

use leptos::view;
use leptos::*;
use mail_send::mail_builder::MessageBuilder;
use serde::Deserialize;

use sqlx::postgres::PgRow;
use sqlx::{FromRow, Row};
use sqlx::{Pool, Postgres};
use time::Instant;

use std::sync::Arc;
use tokio::sync::Mutex;

use tower_http::services::ServeDir;

use jsonwebtoken::{DecodingKey, EncodingKey};
use once_cell::sync::Lazy;
use tokio::io::{AsyncRead, AsyncWrite};
mod components;
use components::*;

mod db_api;
use db_api::*;

mod auth_memes;
use crate::auth_memes::{authorize, check_client, AuthPayload};

type AppState = Arc<Mutex<App>>;

static N_PLANTS: i32 = 9;

static KEYS: Lazy<Keys> = Lazy::new(|| {
    let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    Keys::new(secret.as_bytes())
});

#[tokio::main]
async fn main() -> Result<()> {
    let pool = init_pool(&std::env::var("DATABASE_URL").unwrap()).await?;

    let app_state = Arc::new(Mutex::new(App {
        db_pool: pool,
        //state: Vec::new(),
    }));

    let css_server = ServeDir::new("resources/css");

    let protected_routes = Router::new()
        .route("/", get(index))
        .route("/add-plant", post(post_add_plant))
        .route("/plant-view", get(get_plant_view))
        .route("/add-view", get(get_add_view))
        .route("/sort-by-feed", get(get_sorted_feed_plant_view))
        .route("/sort-by-pot", get(get_sorted_pot_plant_view))
        .route("/sort-by-prune", get(get_sorted_prune_plant_view))
        .route("/update-view", get(get_update_view))
        .route("/update-plant", post(post_update_plant))
        .route("/search-plants", post(search_plants))
        .route("/get-plants-that-need-attention", get(get_plants_attn))
        .layer(middleware::from_fn(check_client))
        .with_state(app_state.clone());

    let unprotected_routes = Router::new()
        .route("/login", get(get_login_page))
        .route("/auth", post(authorize))
        .route("/signup", post(signup_user))
        .route("/login-email", post(login_user))
        .with_state(app_state);

    let app = Router::new()
        .merge(protected_routes)
        .merge(unprotected_routes)
        .nest_service("/css", get_service(css_server));

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

//  #[derive(Debug)]
//  pub struct UserState {
//      user_id: usize,
//      plants: Rc<Vec<Plant>>, //to cache the current list of plants, rather than always hitting db
//  }

#[derive(Clone, Deserialize, Debug, sqlx::FromRow)]
pub struct User {
    pub user_id: i32,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password_hash: String,
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

#[derive(Debug, Deserialize)]
pub struct UserLogin {
    pub email: String,
    pub password: String,
}

async fn index(State(app): State<AppState>, Extension(user_id): Extension<i32>) -> Html<String> {
    let pool = &app.lock().await.db_pool;
    let plants = get_all_plants(pool, user_id, N_PLANTS.to_string())
        .await
        .unwrap();

    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx,
            <Index
                plants=plants
            />
        }
    });

    Html(html)
}

async fn get_login_page(State(_app): State<AppState>) -> Html<String> {
    let html = leptos::ssr::render_to_string(move |cx| {
        view! {cx,
            <NotLoggedInMain/>
        }
    });

    Html(html)
}

async fn login_user(
    State(app): State<AppState>,
    Form(user_login): Form<UserLogin>,
) -> impl IntoResponse {
    let pool = &app.lock().await.db_pool;

    //TODO - if this fails then we should return a 403 as the user hasn't signed up yet or signed
    //up with incorrect email
    let user = db_api::get_user_from_email(pool, user_login.email)
        .await
        .unwrap();

    let is_correct_pw = check_password(&user_login.password, &user.password_hash);
    if !is_correct_pw {
        return Response::builder()
            .status(StatusCode::UNAUTHORIZED)
            .body("Incorrect Password".to_owned())
            .unwrap();
    }

    let auth_payload = AuthPayload {
        client_id: user.user_id.to_string(),
        client_secret: user.password_hash,
    };
    let cookie = get_jwt_cookie_for_new_user(auth_payload);
    let plants = db_api::get_all_plants(pool, user.user_id, 9.to_string())
        .await
        .unwrap_or(Vec::new());

    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx,
            <Index
                plants=plants
            />
        }
    });
    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::SET_COOKIE, cookie)
        .header(header::CONTENT_TYPE, "text/html")
        .body(html)
        .unwrap();
    response
}

async fn signup_user(State(app): State<AppState>, Form(user): Form<User>) -> impl IntoResponse {
    println!("{:?}", user);
    let mut user = user;
    let pool = &app.lock().await.db_pool;
    let pw_hash = auth_memes::hash_password(user.password_hash);

    user.password_hash = pw_hash.clone();

    let user_id = db_api::add_user_to_db(pool, user.clone()).await.unwrap();
    user.user_id = user_id;

    let auth_payload = AuthPayload {
        client_id: user_id.to_string(),
        client_secret: pw_hash.clone(),
    };
    let cookie = get_jwt_cookie_for_new_user(auth_payload);
    let plants = Vec::new();
    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx,
            <Index
                plants=plants
            />
        }
    });

    let response = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "text/html")
        .header(header::SET_COOKIE, cookie)
        .body(html)
        .unwrap();
    response
}

pub async fn get_add_view(
    State(_app): State<AppState>,
    Extension(user_id): Extension<i32>,
) -> Html<String> {
    let html = leptos::ssr::render_to_string(move |cx| {
        view! {cx,
            <AddPlantView
                user_id=user_id
                plant_id=None
                text="Add Plant".into()
            />
        }
    });
    Html(html)
}

pub async fn get_plant_view(
    State(app): State<AppState>,
    Extension(user_id): Extension<i32>,
) -> Html<String> {
    let pool = &app.lock().await.db_pool;
    let plants = get_all_plants(pool, user_id, N_PLANTS.to_string())
        .await
        .unwrap_or(Vec::new());

    let html = leptos::ssr::render_to_string(move |cx| {
        view! {cx,
            <PlantView
                plants=plants
            />
        }
    });
    Html(html)
}

pub async fn get_sorted_feed_plant_view(
    State(app): State<AppState>,
    Extension(user_id): Extension<i32>,
) -> Html<String> {
    let pool = &app.lock().await.db_pool;
    let mut plants = get_all_plants(pool, user_id, N_PLANTS.to_string())
        .await
        .unwrap();
    plants.sort_by(|a, b| {
        let next_a = a.last_fed - time::Duration::days(a.feed_interval as i64);
        let next_b = b.last_fed - time::Duration::days(b.feed_interval as i64);
        next_a.cmp(&next_b)
    });
    let html = leptos::ssr::render_to_string(move |cx| {
        view! {cx,
            <PlantView
                plants=plants
            />
        }
    });
    Html(html)
}

pub async fn get_sorted_pot_plant_view(
    State(app): State<AppState>,
    Extension(user_id): Extension<i32>,
) -> Html<String> {
    let pool = &app.lock().await.db_pool;
    let mut plants = get_all_plants(pool, user_id, N_PLANTS.to_string())
        .await
        .unwrap();
    plants.sort_by(|a, b| {
        let next_a = a.last_potted - time::Duration::days(a.potting_interval as i64);
        let next_b = b.last_potted - time::Duration::days(b.potting_interval as i64);
        next_a.cmp(&next_b)
    });
    let html = leptos::ssr::render_to_string(move |cx| {
        view! {cx,
            <PlantView
                plants=plants
            />
        }
    });
    Html(html)
}

pub async fn get_sorted_prune_plant_view(
    State(app): State<AppState>,
    Extension(user_id): Extension<i32>,
) -> Html<String> {
    let pool = &app.lock().await.db_pool;
    let mut plants = get_all_plants(pool, user_id, N_PLANTS.to_string())
        .await
        .unwrap();
    plants.sort_by(|a, b| {
        let next_a = a.last_pruned - time::Duration::days(a.pruning_interval as i64);
        let next_b = b.last_pruned - time::Duration::days(b.pruning_interval as i64);
        next_a.cmp(&next_b)
    });
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
    Extension(user_id): Extension<i32>,
    Form(plant_id): Form<PlantID>,
) -> Html<String> {
    let pool = &app.lock().await.db_pool;
    //let user_id = user_id.unwrap_or(Extension(1)).0;
    let plant = db_api::get_plant_from_id(pool, user_id, plant_id.plant_id)
        .await
        .unwrap();

    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx,
            <UpdateView
                plant=plant
                user_id=user_id
            />

        }
    });

    Html(html)
}

pub async fn post_update_plant(
    State(app): State<AppState>,
    Extension(user_id): Extension<i32>,
    Form(plant): Form<Plant>,
) -> Html<String> {
    let pool = &app.lock().await.db_pool;
    db_api::update_plant(pool, plant).await.unwrap();
    let plants = get_all_plants(pool, user_id, N_PLANTS.to_string())
        .await
        .unwrap();

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
    Extension(user_id): Extension<i32>,
    Form(search): Form<Search>,
) -> Html<String> {
    let pool = &app.lock().await.db_pool;

    let plants = db_api::search_plants(pool, search.search_string, user_id)
        .await
        .unwrap();

    let html = leptos::ssr::render_to_string(move |cx| {
        view! { cx,
            <PlantView
                plants=plants
            />
        }
    });
    Html(html)
}

pub async fn get_plants_attn(
    State(app): State<AppState>,
    user_id: Option<Extension<i32>>,
) -> Html<String> {
    let pool = &app.lock().await.db_pool;
    let user_id = user_id.unwrap_or(Extension(1)).0;
    let plants = db_api::get_plants_that_need_attention(pool, user_id)
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
        .unwrap_or_else(|_| panic!("Error sending email to: {}", email));
}
