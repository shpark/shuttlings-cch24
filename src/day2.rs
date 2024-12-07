use std::{iter::zip, net::{Ipv4Addr, Ipv6Addr}, str::FromStr};

use axum::{extract::Query, response::IntoResponse};

#[derive(serde::Deserialize)]
pub(super) struct Pack {
    from: Option<String>,
    key: Option<String>,
    to: Option<String>,
}

pub(super) async fn dest(pack: Query<Pack>) -> impl IntoResponse {
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

pub(super) async fn key(pack: Query<Pack>) -> impl IntoResponse {
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

pub(super) async fn dest6(pack: Query<Pack>) -> impl IntoResponse {
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

pub(super) async fn key6(pack: Query<Pack>) -> impl IntoResponse {
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
