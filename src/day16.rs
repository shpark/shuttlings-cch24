use axum::{extract::State, http::{self, HeaderMap, HeaderValue, StatusCode}, response::IntoResponse};
use jwt_simple::{claims::Claims, prelude::{Duration, HS256Key, MACLike}};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
struct AuxData {
    payload: String,
}

pub(super) async fn wrap(
    headers: HeaderMap,
    State(state): State<AppState>,
    payload: String,
) -> impl IntoResponse {
    let mut response_headers: HeaderMap = HeaderMap::new();

    if !headers.get(http::header::CONTENT_TYPE)
        .is_some_and(|v| v == "application/json") {
        return (
            StatusCode::BAD_REQUEST,
            response_headers,
            String::new(),
        )
    }

    let claims = Claims::with_custom_claims(
        AuxData { payload },
        Duration::from_hours(1)
    );
    let jwt = state.jwt_key.authenticate(claims).unwrap();

    response_headers.insert(
        http::header::SET_COOKIE,
        HeaderValue::from_str(format!("gift={}", jwt).as_str()).unwrap(),
    );

    (StatusCode::OK, response_headers, String::new())
}

pub(super) async fn unwrap(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if let Some(cookie) = headers.get(http::header::COOKIE) {
        let parts = cookie.to_str().unwrap().split("=").collect::<Vec<_>>();
        
        if parts[0] == "gift" {
            let jwt = parts[1];

            if let Ok(res) = state.jwt_key.verify_token::<AuxData>(jwt, None) {
                return (
                    StatusCode::OK,
                    res.custom.payload,
                )
            }
        }
    }

    (StatusCode::BAD_REQUEST, String::new())
}

#[cfg(test)]
mod test {
    use jwt_simple::{claims::Claims, prelude::{Base64, Duration, HS256Key, MACLike}, reexports::ct_codecs::Encoder};


    #[test]
    fn test_jwt_simple() {
        let key = HS256Key::generate();

        let base64_encoding = Base64::encode_to_string(key.to_bytes());

        println!("{}", base64_encoding.unwrap());

        let claims = Claims::create(Duration::from_hours(2));

        let jwt = key.authenticate(claims).unwrap();

        println!("{}", jwt);
    }
}