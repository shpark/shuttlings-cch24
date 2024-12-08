use axum::{http::{HeaderMap, StatusCode}, response::IntoResponse};
use cargo_manifest::{Manifest, MaybeInherited};
use serde::Deserialize;
use toml::Value;

const INVALID_MANIFEST: &str = "Invalid manifest";

const MAGIC_WORD: &str = "Christmas 2024";
const MAGIC_WORD_NOT_FOUND: &str = "Magic keyword not provided";

#[derive(PartialEq, Debug, serde::Deserialize)]
struct Order {
    item: String,
    quantity: u32,
}

#[derive(Debug, serde::Deserialize)]
struct Orders(#[serde(deserialize_with = "deserialize_orders")] Vec<Order>);

#[derive(Debug, serde::Deserialize)]
struct Metadata {
    orders: Option<Orders>,
}

fn deserialize_orders<'de, D>(
    deserializer: D,
) -> Result<Vec<Order>, D::Error>
where
    D: serde::de::Deserializer<'de>
{
    let raw_orders: Vec<Value> = serde::Deserialize::deserialize(deserializer)?;

    let mut valid_orders = Vec::new();

    for raw_order in raw_orders {
        if let Ok(order) = Order::deserialize(raw_order) {
            valid_orders.push(order);
        }
    }

    Ok(valid_orders)
}

pub(super) async fn manifest(
    headers: HeaderMap,
    body: String,
) -> impl IntoResponse {
    if let Some(content_type) = headers.get("Content-Type") {
        if content_type.as_ref() != "application/toml".as_bytes() {
            return (StatusCode::BAD_REQUEST, String::new()); // FIXME?
        }
    }

    if let Ok(manifest) = Manifest::<Metadata>::from_slice_with_metadata(body.as_bytes()) {
        if let Some(package) = manifest.package {
            // Test if the manifest contains the magic word (FIXME: Ugly)
            match package.keywords {
                Some(MaybeInherited::Local(keywords)) => {
                    if keywords.iter().all(|keyword| keyword != MAGIC_WORD) {
                        return (StatusCode::BAD_REQUEST, String::from(MAGIC_WORD_NOT_FOUND));
                    }
                },
                _ => {
                    return (StatusCode::BAD_REQUEST, String::from(MAGIC_WORD_NOT_FOUND));
                },
            }

            if let Some(metadata) = package.metadata {
                if let Some(orders) = metadata.orders {
                    let orders = orders.0.iter()
                        .map(|order| format!("{}: {}", order.item, order.quantity))
                        .collect::<Vec<_>>();

                    if orders.len() > 0 {
                        return (StatusCode::OK, orders.join("\n"));
                    } 
                }
            }
        }
    } else {
        return (StatusCode::BAD_REQUEST, String::from(INVALID_MANIFEST));
    }

    (StatusCode::NO_CONTENT, String::new())
}

#[cfg(test)]
mod test {
    use super::*;

    use cargo_manifest::Manifest;

    #[test]
    fn test_cargo_manifest_with_metadata() {
        let manifest = Manifest::<super::Metadata>::from_slice_with_metadata(r#"
[package]
name = "not-a-gift-order"
authors = ["Not Santa"]
keywords = ["Christmas 2024"]

[[package.metadata.orders]]
item = "Toy car"
quantity = 2

[[package.metadata.orders]]
item = "Lego brick"
quantity = "hello, world"
       "#.as_bytes())
        .unwrap();

        assert_eq!(
            manifest.package.unwrap().metadata.unwrap().orders.unwrap().0,
            vec![Order { item: String::from("Toy car"), quantity: 2 }]
        );
    }
} 