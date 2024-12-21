use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse, Json};
use serde::{Serialize, Deserialize};
use sqlx::{prelude::FromRow, query, query_as, types::{chrono::{DateTime, Utc}, Uuid}};

use crate::AppState;

#[derive(Deserialize)]
pub(super) struct NewQuote {
    author: String,
    quote: String,
}

#[derive(Debug, Serialize, FromRow)]
pub(super) struct Quote {
    id: Uuid,
    author: String,
    quote: String,
    created_at: DateTime<Utc>,
    version: i32,
}

// POST /19/reset: Clear the quotes table in the database.
pub(super) async fn reset(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match query("DELETE FROM quotes;").execute(&state.pool).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

// GET /19/cite/{id}: Respond with the quote of the given ID.
// Use 404 Not Found if a quote with the ID does not exist.
pub(super) async fn cite(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match query_as::<_, Quote>("SELECT * FROM quotes WHERE id = $1;")
    .bind(id)
    .fetch_one(&state.pool)
    .await {
        Ok(quote) => {
            Ok((
                StatusCode::OK,
                serde_json::to_string(&quote).unwrap(),
            ))
        }
        Err(_) => Err((StatusCode::NOT_FOUND, String::new())),
    }
}

// DELETE /19/remove/{id}: Delete and respond with the quote of the given ID.
// Same 404 logic as above.
pub(super) async fn remove(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match query_as::<_, Quote>(r#"
DELETE FROM quotes
WHERE id = $1
RETURNING *;
    "#)
    .bind(id)
    .fetch_one(&state.pool)
    .await {
        Ok(quote) => Ok((
            StatusCode::OK,
            serde_json::to_string(&quote).unwrap(),
        )),
        Err(_) => Err((StatusCode::NOT_FOUND, String::new())),
    }
}

// PUT /19/undo/{id}: Update the author and text, and increment the version
// number of the quote of the given ID. Respond with the updated quote.
// Same 404 logic as above.
pub(super) async fn undo(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(quote): Json<NewQuote>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match query_as::<_, Quote>(r#"
UPDATE quotes
SET 
    author = $1,
    quote = $2,
    version = version + 1
WHERE id = $3
RETURNING id, author, quote, created_at, version;
    "#)
    .bind(quote.author)
    .bind(quote.quote)
    .bind(id)
    .fetch_one(&state.pool)
    .await {
        Ok(quote) => {
            Ok((
                StatusCode::OK,
                serde_json::to_string(&quote).unwrap(),
            ))
        },
        Err(_) => {
            Err((StatusCode::NOT_FOUND, String::new()))
        },
    }
}

// POST /19/draft: Add a quote with a random UUID v4. Respond with the quote
// and 201 Created.
pub(super) async fn draft(
    State(state): State<AppState>,
    Json(quote): Json<NewQuote>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    match query_as::<_, Quote>(r#"
INSERT INTO quotes (id, author, quote)
VALUES ($1, $2, $3)
RETURNING id, author, quote, created_at, version;
    "#)
    .bind(Uuid::new_v4())
    .bind(quote.author)
    .bind(quote.quote)
    .fetch_one(&state.pool)
    .await {
        Ok(quote) => Ok((
            StatusCode::CREATED,
            serde_json::to_string(&quote).unwrap(),
        )),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            String::new(),
        )),
    }
}
