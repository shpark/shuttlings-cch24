use std::{fmt, str::FromStr};

use axum::{extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, Json};
use rand::{distributions::Alphanumeric, Rng};
use serde::{de, Deserialize, Deserializer, Serialize};
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

#[derive(Debug, Deserialize)]
pub(super) struct Params {
    #[serde(default, deserialize_with="empty_string_as_none")]
    token: Option<String>,
}

// See https://github.com/tokio-rs/axum/blob/main/examples/query-params-with-empty-strings/src/main.rs
//
// Serde deserialization decorator to map empty Strings to None
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

#[derive(Serialize)]
struct Quotes {
    quotes: Vec<Quote>,
    page: i32,
    next_token: Option<String>,
}

pub(super) async fn list(
    State(state): State<AppState>,
    Query(params): Query<Params>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    let page = if let Some(token) = &params.token {
        if let Some(&offset) = state.token_to_offset.read().await.get(token) {
            offset
        } else {
            // A token was provided, but it is unknown or badly formatted.
            return Err(StatusCode::BAD_REQUEST);
        }
    } else {
        1
    };

    let mut token_to_offset = state.token_to_offset.write().await;

    // FIXME(inefficiency): To determine whether we have to issue a token or
    // not, we fetch up to 4 rows and see if the fourth row is present. If so,
    // we should issue a new token, otherwise we shouldn't.
    match query_as::<_, Quote>(r#"
SELECT * FROM quotes ORDER BY created_at ASC LIMIT 4 OFFSET ($1 - 1) * 3;
    "#)
    .bind(page)
    .fetch_all(&state.pool)
    .await {
        Ok(mut quotes) => {
            let next_token = if quotes.len() < 4 {
                None
            } else {
                let mut rng = rand::thread_rng();
                let next_token = (0..16)
                    .map(|_| rng.sample(Alphanumeric) as char)
                    .collect::<String>();

                token_to_offset.insert(next_token.clone(), page + 1);

                Some(next_token)
            };

            // We only have to return the first 3 quotes...
            quotes.pop();

            let quotes = Quotes {
                quotes,
                page,
                next_token,
            };

            Ok((
                StatusCode::OK,
                serde_json::to_string(&quotes).unwrap(),
            ))
        },

        // No quotes fetched?
        Err(_e) => Err(StatusCode::NOT_FOUND),
    }
}
