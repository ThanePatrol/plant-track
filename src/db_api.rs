use sqlx::postgres::PgPoolOptions;
use sqlx::{Pool, Postgres, Row};

use super::Plant;
use anyhow::Result;

pub async fn init_pool(db_url: &str) -> Result<Pool<Postgres>> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(db_url)
        .await?;
    Ok(pool)
}

pub async fn get_all_plants(
    pool: &Pool<Postgres>,
    user_id: i32,
    n_records: String,
) -> Result<Vec<Plant>> {
    let limit = match n_records.parse::<i32>().unwrap_or(-1) {
        -1 => None,
        x => Some(x),
    };

    let rows = sqlx::query_as(
        r#"
        SELECT * FROM plants
        WHERE user_id = $1
        LIMIT $2
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn add_plant_to_db(pool: &Pool<Postgres>, plant: Plant) -> Result<i32> {
    let result = sqlx::query(
        "INSERT INTO plants (user_id, botanical_name, common_name, last_fed, feed_interval, \
            last_potted, potting_interval, last_pruned, pruning_interval)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING plant_id
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
    .fetch_one(pool)
    .await?;
    Ok(result.get::<i32, _>("plant_id"))
}

pub async fn get_plants_that_need_attention(
    pool: &Pool<Postgres>,
    user_id: i32,
) -> Result<Vec<Plant>> {
    let rows = sqlx::query_as(
        r#"

        SELECT 
          * 
        FROM 
          plants 
        WHERE 
          user_id = $1
          AND (
            last_fed < CURRENT_DATE - feed_interval * INTERVAL '1 day'
            OR 
            last_potted < CURRENT_DATE - potting_interval * INTERVAL '1 day'
            OR 
            last_pruned < CURRENT_DATE - pruning_interval * INTERVAL '1 day'
          );
    "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_plant_from_id(
    pool: &Pool<Postgres>,
    user_id: i32,
    plant_id: i32,
) -> Result<Plant> {
    let row = sqlx::query_as(
        r#"
        SELECT * FROM plants WHERE user_id = $1 AND plant_id = $2;
        "#,
    )
    .bind(user_id)
    .bind(plant_id)
    .fetch_one(pool)
    .await?;
    Ok(row)
}

pub async fn update_plant(pool: &Pool<Postgres>, plant: Plant) -> Result<()> {
    let _ = sqlx::query(
        r#"
        UPDATE plants
        SET botanical_name = $1, common_name = $2, last_fed = $3, feed_interval = $4, 
        last_potted = $5, potting_interval = $6, last_pruned = $7, pruning_interval = $8
        WHERE plant_id = $9
        "#,
    )
    .bind(plant.botanical_name)
    .bind(plant.common_name)
    .bind(plant.last_fed)
    .bind(plant.feed_interval)
    .bind(plant.last_potted)
    .bind(plant.potting_interval)
    .bind(plant.last_pruned)
    .bind(plant.pruning_interval)
    .bind(plant.plant_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn search_plants(
    pool: &Pool<Postgres>,
    search: String,
    user_id: i32,
) -> Result<Vec<Plant>> {
    let search = format!("%{}%", search);
    let rows = sqlx::query_as(
        r#"
        SELECT * FROM plants
        WHERE user_id = $1 AND (botanical_name ILIKE $2 OR common_name ILIKE $3) 
        "#,
    )
    .bind(user_id)
    .bind(search.clone())
    .bind(search)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

pub async fn get_user_email(pool: &Pool<Postgres>, user_id: i32) -> Result<String> {
    let email = sqlx::query("SELECT email FROM users WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .unwrap()
        .get::<String, _>("email");
    Ok(email)
}
