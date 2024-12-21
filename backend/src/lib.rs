extern crate core;

pub mod schema;
pub mod models;
pub mod utils;
pub mod data;
pub mod errors;
pub mod middleware;

use diesel::prelude::*;
use dotenv::dotenv;
use std::env;
use crate::data::DBClient;
use crate::utils::config::Config;

#[derive(Debug, Clone)]
pub struct AppState {
    pub env: Config,
    pub db_client: DBClient,
}

pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}