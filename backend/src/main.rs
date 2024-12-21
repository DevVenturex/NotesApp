use axum::http::header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use axum::{Extension, Router};
use diesel::PgConnection;
use diesel::r2d2::ConnectionManager;
use dotenv::dotenv;
use tower_http::cors::CorsLayer;
use tracing_subscriber::filter::LevelFilter;
use backend::AppState;
use backend::data::db::{DBClient, PgPool};
use backend::utils::config::Config;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::DEBUG)
        .init();

    dotenv().ok();

    let config = Config::init();
    let manager = ConnectionManager::<PgConnection>::new(&config.database_url);
    let pool = PgPool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::PUT]);

    let db_client = DBClient::new(pool);
    let app_state = AppState {
        env: config.clone(),
        db_client: db_client.clone(),
    };

    let app = Router::new()
        .layer(Extension(app_state))
        .layer(cors.clone());

    println!("Starting server on http://localhost:{}", config.port);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", &config.port)).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
