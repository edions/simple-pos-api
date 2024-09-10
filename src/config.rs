use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Debug)]
pub struct Config {
    pub pool: PgPool,
    pub address: String,
}

pub async fn init_config() -> Config {
    dotenvy::dotenv().expect("Unable to access .env file");

    let server_address = std::env::var("SERVER_ADDRESS").unwrap();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found in env file");

    let db_pool = PgPoolOptions::new()
        .max_connections(64)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("Can't connect to database");

    Config {
        pool: db_pool,
        address: server_address,
    }
}