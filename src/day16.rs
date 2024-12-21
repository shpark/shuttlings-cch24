use axum::{extract::State, http::{self, HeaderMap, HeaderValue, StatusCode}, response::IntoResponse};
use jwt_simple::{claims::Claims, prelude::{Duration, MACLike, RS256PublicKey, RS512PublicKey, RSAPublicKeyLike}, token::Token, JWTError};
use serde_json::Value;

use crate::AppState;

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

    let claims = Claims::with_custom_claims::<Value>(
        serde_json::from_str(payload.as_str()).unwrap(),
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

            if let Ok(res) = state.jwt_key.verify_token::<Value>(jwt, None) {
                return (
                    StatusCode::OK,
                    serde_json::to_string(&res.custom).unwrap(),
                )
            }
        }
    }

    (StatusCode::BAD_REQUEST, String::new())
}

pub(super) async fn decode(
    State(state): State<AppState>,
    jwt: String,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if let Ok(metadata) = Token::decode_metadata(jwt.as_str()) {
        let claims = match metadata.algorithm() {
            // FIXME: I know it is perhaps too dumb to initialize
            // RS(256|512)PublicKey for each request, but the API
            // is not flexible enough, unfortunately. For example,
            // if there were ways to create RS(256|512)PublicKey
            // from RSAPublicKey, then we could keep RSAPublicKey
            // to AppStatus and reuse it. But jwt_simple API currently
            // does not allow that.
            "RS256" => {
                RS256PublicKey::from_pem(state.santa_public_pem).unwrap()
                    .verify_token::<Value>(jwt.as_str(), None)
            },
            "RS512" => {
                RS512PublicKey::from_pem(state.santa_public_pem).unwrap()
                    .verify_token::<Value>(jwt.as_str(), None)
            },
            _ => return Err((StatusCode::BAD_REQUEST, String::new())),
        };

        match claims {
            Ok(claims) => Ok((
                StatusCode::OK,
                serde_json::to_string(&claims.custom).unwrap(),
            )),
            Err(e) => {
                match e.downcast::<JWTError>() {
                    Ok(JWTError::InvalidSignature) => Err((
                        StatusCode::UNAUTHORIZED,
                        String::new()
                    )),
                    _ => Err((StatusCode::BAD_REQUEST, String::new()))
                }
            }
        }
    } else {
        Err((
            StatusCode::BAD_REQUEST,
            String::new(),
        ))
    }
}
