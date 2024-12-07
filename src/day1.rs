use axum::{body::Body, http::{Response, StatusCode}, response::IntoResponse};

pub(super) async fn hello_world() -> &'static str {
    "Hello, bird!"
}

pub(super) async fn seek() -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "https://www.youtube.com/watch?v=9Gc4QTqslN4")
        .body(Body::empty())
        .unwrap()
}
