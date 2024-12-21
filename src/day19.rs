use axum::{extract::Path, http::StatusCode, response::IntoResponse};
use sqlx::types::Uuid;

// "/19/reset", post(day19::reset))
pub(super) async fn reset() -> impl IntoResponse {
    (StatusCode::OK, String::new())
}

// "/19/cite/:id", get(day19::cite))
pub(super) async fn cite(
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    (StatusCode::OK, String::new())
}

// "/19/remove/:id", delete(day19::remove))
pub(super) async fn remove(
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    (StatusCode::OK, String::new())
}

// "/19/undo/:id", put(day19::undo))
pub(super) async fn undo(
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    (StatusCode::OK, String::new())
}

// "/19/draft", post(day19::draft))
pub(super) async fn draft() -> impl IntoResponse {
    (StatusCode::OK, String::new())
}