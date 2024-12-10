use axum::{routing::{get, post}, Router};

mod day1;
mod day2;
mod day5;
mod day9;

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
        // .route("/9/milk", get(day9::milk))
        .route("/", get(day1::hello_world))
        .with_state(day9::MilkBucket::new());

    Ok(router.into())
}
