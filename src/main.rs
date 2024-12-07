use axum::{routing::get, Router};

mod day1;
mod day2;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/-1/seek", get(day1::seek))
        .route("/2/dest", get(day2::dest))
        .route("/2/key", get(day2::key))
        .route("/2/v6/dest", get(day2::dest6))
        .route("/2/v6/key", get(day2::key6))
        .route("/", get(day1::hello_world));

    Ok(router.into())
}
