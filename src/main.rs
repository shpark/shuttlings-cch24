use std::sync::Arc;

use axum::{routing::{get, post}, Router};
use day12::Board;
use day9::MilkBucket;
use tokio::sync::RwLock;

mod day1;
mod day2;
mod day5;
mod day9;
mod day12;

#[derive(Clone)]
struct AppState {
    milk_bucket: Arc<RwLock<day9::MilkBucket>>,
    board: Arc<RwLock<day12::Board<4, 4>>>,
}

impl AppState {
    fn new() -> Self {
        AppState {
            milk_bucket: Arc::new(RwLock::new(MilkBucket::new())),
            board: Arc::new(RwLock::new(Board::new())),
        }
    }
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
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
        .route("/", get(day1::hello_world))
        .with_state(AppState::new());

    Ok(router.into())
}
