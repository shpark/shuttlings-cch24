use std::sync::Arc;

use axum::{routing::{delete, get, post, put}, Router};
use day12::Board;
use day9::MilkBucket;
use jwt_simple::prelude::HS256Key;
use sqlx::PgPool;
use tokio::sync::RwLock;
use tower_http::services::ServeDir;

mod day1;
mod day2;
mod day5;
mod day9;
mod day12;
mod day16;
mod day19;
mod day23;

#[derive(Clone)]
struct AppState {
    milk_bucket: Arc<RwLock<day9::MilkBucket>>,
    board: Arc<RwLock<day12::Board<4>>>,
    jwt_key: HS256Key,
    santa_public_pem: &'static str,
    pool: PgPool,
}

impl AppState {
    fn with_pool(pool: PgPool) -> Self {
        AppState {
            milk_bucket: Arc::new(RwLock::new(MilkBucket::new())),
            board: Arc::new(RwLock::new(Board::new())),
            jwt_key: HS256Key::generate(), // ¯\_(ツ)_/¯
            santa_public_pem: include_str!("./day16_santa_public_key.pem"),
            pool,
        }
    }
}

#[shuttle_runtime::main]
async fn main(
    #[shuttle_shared_db::Postgres(
        local_uri = "postgres://postgres:very_secure_password@localhost:5432/postgres"
    )] pool: PgPool,
) -> shuttle_axum::ShuttleAxum {
    // TODO
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let router = Router::new()
        .route("/-1/seek", get(day1::seek))
        .route("/2/dest", get(day2::dest))
        .route("/2/key", get(day2::key))
        .route("/2/v6/dest", get(day2::dest6))
        .route("/2/v6/key", get(day2::key6))
        .route("/5/manifest", post(day5::manifest))
        .route("/9/milk", post(day9::milk))
        .route("/9/refill", post(day9::refill))
        .route("/12/board", get(day12::board))
        .route("/12/reset", post(day12::reset))
        .route("/12/place/:team/:column", post(day12::place))
        .route("/16/wrap", post(day16::wrap))
        .route("/16/unwrap", get(day16::unwrap))
        .route("/16/decode", post(day16::decode))
        .route("/19/reset", post(day19::reset))
        .route("/19/cite/:id", get(day19::cite))
        .route("/19/remove/:id", delete(day19::remove))
        .route("/19/undo/:id", put(day19::undo))
        .route("/19/draft", post(day19::draft))
        .route("/", get(day1::hello_world))
        .route("/23/star", get(day23::star))
        .route("/23/present/:color", get(day23::present))
        .route("/23/ornament/:state/:n", get(day23::ornament))
        .with_state(AppState::with_pool(pool))
        .nest_service("/assets", ServeDir::new("assets"));

    Ok(router.into())
}
