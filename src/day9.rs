use std::sync::Arc;

use axum::{extract::State, http::{HeaderMap, StatusCode}, response::IntoResponse, Json};
use leaky_bucket::RateLimiter;
use tokio::{sync::RwLock, time::Duration};

const MILK_WITHDRAWN: &str = "Milk withdrawn\n";

const NO_MILK_AVAILABLE: &str = "No milk available\n";

#[derive(Clone)]
pub(super) struct MilkBucket(Arc<RwLock<RateLimiter>>);

impl MilkBucket {
    pub(super) fn new() -> MilkBucket {
        MilkBucket(Arc::new(RwLock::new(RateLimiter::builder()
                .initial(5)
                .max(5)
                .interval(Duration::from_millis(1000))
                .build())))
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
    State(milk_bucket): State<MilkBucket>,
    milk_unit: Option<Json<MilkUnit>>,
) -> impl IntoResponse {
    if !milk_bucket.0.read().await.try_acquire(1) {
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
    State(milk_bucket): State<MilkBucket>,
) -> impl IntoResponse {
    let mut old_bucket = milk_bucket.0.write().await;

    let _ = std::mem::replace(
        &mut *old_bucket,
        RateLimiter::builder()
            .initial(5)
            .max(5)
            .interval(Duration::from_millis(1000))
            .build()
    );

    StatusCode::OK
}