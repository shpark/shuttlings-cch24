use axum::{extract::State, http::{HeaderMap, StatusCode}, response::IntoResponse, Json};
use leaky_bucket::RateLimiter;
use tokio::time::Duration;

use crate::AppState;

const MILK_WITHDRAWN: &str = "Milk withdrawn\n";

const NO_MILK_AVAILABLE: &str = "No milk available\n";

pub(super) struct MilkBucket(RateLimiter);

impl MilkBucket {
    pub(super) fn new() -> MilkBucket {
        MilkBucket(RateLimiter::builder()
                .initial(5)
                .max(5)
                .interval(Duration::from_millis(1000))
                .build())
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) struct MilkUnit {
    #[serde(skip_serializing_if = "Option::is_none")]
    liters: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gallons: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    litres: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pints: Option<f32>,
}

pub(super) async fn milk(
    headers: HeaderMap,
    State(state): State<AppState>,
    milk_unit: Option<Json<MilkUnit>>,
) -> impl IntoResponse {
    if !state.milk_bucket.read().await.0.try_acquire(1) {
        return (
            StatusCode::TOO_MANY_REQUESTS,
            String::from(NO_MILK_AVAILABLE),
        );
    }

    if !headers.get("Content-Type").is_some_and(|v| v == "application/json") {
        return (
                StatusCode::OK,
                String::from(MILK_WITHDRAWN),
        );
    }

    if let Some(u) = milk_unit {
        match (u.gallons, u.liters, u.litres, u.pints) {
            (Some(gallons), None, None, None) => Some(MilkUnit {
                gallons: None,
                liters: Some(gallons * 3.78541),
                litres: None,
                pints: None,
            }),
            (None, Some(liters), None, None) => Some(MilkUnit {
                gallons: Some(liters / 3.78541),
                liters: None,
                litres: None,
                pints: None,
            }),
            (None, None, Some(litres), None) => Some(MilkUnit {
                gallons: None,
                liters: None,
                litres: None,
                pints: Some(litres * 1.7598),
            }),
            (None, None, None, Some(pints)) => Some(MilkUnit {
                gallons: None,
                liters: None,
                litres: Some(pints / 1.7598),
                pints: None,
            }),
            _ => None,
        }
        .map_or(
            (StatusCode::BAD_REQUEST, String::new()),
            |milk_unit| (StatusCode::OK, serde_json::to_string(&milk_unit).unwrap())
        )
    } else {
        (StatusCode::BAD_REQUEST, String::new())
    }
}

pub(super) async fn refill(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let _ = std::mem::replace(
        &mut state.milk_bucket.write().await.0,
        RateLimiter::builder()
            .initial(5)
            .max(5)
            .interval(Duration::from_millis(1000))
            .build()
    );

    StatusCode::OK
}