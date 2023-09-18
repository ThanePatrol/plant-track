use sqlx::postgres::{PgPoolOptions, PgQueryResult, PgRow};
use sqlx::{Pool, Postgres, Row};

use super::{App, AppState, Plant};
use anyhow::Result;

pub async fn init_pool(db_url: &str) -> Result<Pool<Postgres>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;
    Ok(pool)
}

pub async fn get_all_plants(pool: &Pool<Postgres>, user_id: i32) -> Result<Vec<Plant>> {
    let rows = sqlx::query_as(r#"SELECT * FROM plants WHERE user_id = $1"#)
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    Ok(rows)
}

pub async fn add_plant_to_db(pool: &Pool<Postgres>, plant: Plant) -> Result<PgQueryResult> {
    let result = sqlx::query(
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
    .await?;
    Ok(result)
}

//todo add validation that checks if user_id is the same as what was submitted in the form
