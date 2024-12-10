use std::sync::Arc;

use axum::{body::Body, extract::{self, State}, http::{method, HeaderMap, Method, StatusCode}, response::IntoResponse};
use leaky_bucket::RateLimiter;
use tokio::time::Duration;

// use leaky_bucket::RateLimiter;
// use tokio::time::Duration;
// 
// let limiter = RateLimiter::builder()
//     .initial(10)
//     .interval(Duration::from_millis(100))
//     .build();
// 
// // This is instantaneous since the rate limiter starts with 10 tokens to
// // spare.
// limiter.acquire(10).await;
// 
// // This however needs to core switch and wait for a while until the desired
// // number of tokens is available.
// limiter.acquire(3).await;

const MILK_WITHDRAWN: &str = "Milk withdrawn\n";

const NO_MILK_AVAILABLE: &str = "No milk available\n";

#[derive(Clone)]
pub(super) struct MilkBucket {
    bucket: Arc<RateLimiter>,
}

impl MilkBucket {
    pub(super) fn new() -> MilkBucket {
        MilkBucket {
            bucket: Arc::new(RateLimiter::builder()
                .initial(5)
                .max(5)
                .interval(Duration::from_millis(1000))
                .build()),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
struct MilkUnit {
    liters: Option<f32>,
    gallons: Option<f32>,
}

pub(super) async fn milk(
    method: Method,
    headers: HeaderMap,
    body: String,
    milk_bucket: State<MilkBucket>,
) -> impl IntoResponse {
    if matches!(method, Method::POST) ||
        (matches!(method, Method::GET) &&
         !matches!(
            headers.get("Content-Type"), 
            Some(x) if x == "application/json"
          )) {
        return if milk_bucket.0.bucket.try_acquire(1) {
            (
                StatusCode::OK,
                String::from(MILK_WITHDRAWN),
            )
        } else {
            (
                StatusCode::TOO_MANY_REQUESTS,
                String::from(NO_MILK_AVAILABLE),
            )
        };
    }

    if let Ok(milk_unit) = serde_json::from_str::<MilkUnit>(&body) {
        match (milk_unit.gallons, milk_unit.liters) {
            (Some(gallons), None) => Some(MilkUnit {
                gallons: None,
                liters: Some(gallons * 3.78541)
            }),
            (None, Some(liters)) => Some(MilkUnit {
                gallons: Some(liters / 3.78541),
                liters: None,
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
