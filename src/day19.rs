use axum::{extract::Path, http::StatusCode, response::IntoResponse};
use sqlx::types::Uuid;

pub(super) async fn reset() -> impl IntoResponse {
    (StatusCode::OK, String::new())
}

pub(super) async fn cite(
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    (StatusCode::OK, String::new())
}

pub(super) async fn remove(
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    (StatusCode::OK, String::new())
}

pub(super) async fn undo(
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    (StatusCode::OK, String::new())
}

pub(super) async fn draft() -> impl IntoResponse {
    (StatusCode::OK, String::new())
}