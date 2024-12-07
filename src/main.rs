use core::panic;
use std::{fmt, iter::zip, net::{Ipv4Addr, Ipv6Addr}, str::FromStr};

use axum::{body::Body, extract::Query, http::StatusCode, response::{IntoResponse, Response}, routing::get, Router};

async fn hello_world() -> &'static str {
    "Hello, bird!"
}

async fn seek() -> impl IntoResponse{
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", "https://www.youtube.com/watch?v=9Gc4QTqslN4")
        .body(Body::empty())
        .unwrap()
}

#[derive(serde::Deserialize)]
struct Pack {
    from: Option<String>,
    key: Option<String>,
    to: Option<String>,
}

async fn dest(pack: Query<Pack>) -> impl IntoResponse {
    let from = Ipv4Addr::from_str(pack.from.as_ref().unwrap()).unwrap();
    let key = Ipv4Addr::from_str(pack.key.as_ref().unwrap()).unwrap();

    let to = zip(from.octets(), key.octets())
        .map(|(p, q)| p.wrapping_add(q))
        .collect::<Vec<_>>();

    match to.try_into() {
        Ok(bytes) => Ipv4Addr::from_bits(u32::from_be_bytes(bytes)).to_string(),
        _ => panic!("¯\\_(ツ)_/¯")
    }
}

async fn key(pack: Query<Pack>) -> impl IntoResponse {
    let from = Ipv4Addr::from_str(pack.from.as_ref().unwrap()).unwrap();
    let to = Ipv4Addr::from_str(pack.to.as_ref().unwrap()).unwrap();

    let key = zip(to.octets(), from.octets())
        .map(|(p, q)| p.wrapping_sub(q))
        .collect::<Vec<_>>();

    match key.try_into() {
        Ok(bytes) => Ipv4Addr::from_bits(u32::from_be_bytes(bytes)).to_string(),
        _ => panic!("¯\\_(ツ)_/¯")
    }
}

async fn dest6(pack: Query<Pack>) -> impl IntoResponse {
    let from = Ipv6Addr::from_str(pack.from.as_ref().unwrap()).unwrap();
    let key = Ipv6Addr::from_str(pack.key.as_ref().unwrap()).unwrap();

    let to = zip(from.octets(), key.octets())
        .map(|(p, q)| p ^ q)
        .collect::<Vec<_>>();

    match to.try_into() {
        Ok(bytes) => Ipv6Addr::from_bits(u128::from_be_bytes(bytes)).to_string(),
        _ => panic!("¯\\_(ツ)_/¯")
    }
}

async fn key6(pack: Query<Pack>) -> impl IntoResponse {
    let from = Ipv6Addr::from_str(pack.from.as_ref().unwrap()).unwrap();
    let to = Ipv6Addr::from_str(pack.to.as_ref().unwrap()).unwrap();

    let key = zip(to.octets(), from.octets())
        .map(|(p, q)| p ^ q)
        .collect::<Vec<_>>();

    match key.try_into() {
        Ok(bytes) => Ipv6Addr::from_bits(u128::from_be_bytes(bytes)).to_string(),
        _ => panic!("¯\\_(ツ)_/¯")
    }
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/-1/seek", get(seek))
        .route("/2/dest", get(dest))
        .route("/2/key", get(key))
        .route("/2/v6/dest", get(dest6))
        .route("/2/v6/key", get(key6))
        .route("/", get(hello_world));

    Ok(router.into())
}
