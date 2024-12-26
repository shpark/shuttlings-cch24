use std::io::{Error, ErrorKind};

use askama::Template;
use axum::{extract::{Multipart, Path}, http::StatusCode, response::{Html, IntoResponse}};
use serde::Deserialize;

enum PresentColor {
    Red,
    Blue,
    Purple,
}

impl PresentColor {
    fn next(&self) -> Self {
        match self {
            PresentColor::Red => PresentColor::Blue,
            PresentColor::Blue => PresentColor::Purple,
            PresentColor::Purple => PresentColor::Red,
        }
    }
}

impl std::fmt::Display for PresentColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            PresentColor::Red => "red",
            PresentColor::Blue => "blue",
            PresentColor::Purple => "purple",
        })
    }
}

impl From<PresentColor> for PresentTemplateInput {
    fn from(value: PresentColor) -> Self {
        PresentTemplateInput {
            curr: value.to_string(),
            next: value.next().to_string(),
        }
    }
}

#[derive(Template)]
#[template(path = "../templates/present.html")]
struct PresentTemplateInput {
    curr: String,
    next: String,
}

impl TryFrom<&str> for PresentColor {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "red" => Ok(Self::Red),
            "blue" => Ok(Self::Blue),
            "purple" => Ok(Self::Purple),
            _ => Err("meh"),
        }
    }
}

enum OrnamentState {
    On,
    Off,
}

impl OrnamentState {
    fn next(&self) -> Self {
        match self {
            OrnamentState::On => OrnamentState::Off,
            OrnamentState::Off => OrnamentState::On,
        }
    }
}

impl TryFrom<&str> for OrnamentState {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "on" => Ok(Self::On),
            "off" => Ok(Self::Off),
            _ => Err("ding")
        }
    }
}

impl std::fmt::Display for OrnamentState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            OrnamentState::On => "on",
            OrnamentState::Off => "off",
        })
    }
}

#[derive(Template)]
#[template(path = "../templates/ornament.html")]
struct Ornament {
    n: String,
    state: String,
    next_state: OrnamentState,
}

pub(super) async fn star() -> impl IntoResponse {
    Html::from("<div id=\"star\" class=\"lit\"></div>")
}

pub(super) async fn present(
    Path(color): Path<String>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if let Ok(color) = PresentColor::try_from(color.as_str()) {
        Ok(Html::from(PresentTemplateInput::from(color).render().unwrap()))
    } else {
        Err(StatusCode::IM_A_TEAPOT)
    }
}

pub(super) async fn ornament(
    Path((state, n)): Path<(String, String)>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    if let Ok(state) = OrnamentState::try_from(state.as_str()) {
        let next_state = state.next();

        // XXX: meh
        let state = match state {
            OrnamentState::On => String::from(" on"),
            _ => String::new(),
        };

        let ornament = Ornament { n, state, next_state };

        Ok(Html::from(ornament.render().unwrap()))
    } else {
        Err(StatusCode::IM_A_TEAPOT)
    }
}

#[derive(Deserialize, Debug)]
struct Package {
    checksum: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Lockfile {
    package: Vec<Package>,
}

impl TryFrom<String> for ChecksumFragment {
    type Error = Box<dyn std::error::Error>;

    fn try_from(checksum: String) -> Result<Self, Self::Error> {
        if checksum.len() < 6 + 2 + 2 {
            return Err(
                Box::new(Error::new(ErrorKind::InvalidInput, "ding"))
            );
        }

        match u32::from_str_radix(&checksum[0..6], 16) {
            Ok(_) => {
                let color = String::from(&checksum[0..6]);
                let top = u8::from_str_radix(&checksum[6..8], 16)?;
                let left = u8::from_str_radix(&checksum[8..10], 16)?;

                Ok(ChecksumFragment { color, top, left })
            },
            Err(e) => Err(Box::new(e)),
        }
    }
}

#[derive(Template)]
#[template(path = "../templates/checksum.html")]
struct ChecksumFragment {
    color: String,
    top: u8,
    left: u8,
}

pub(super) async fn lockfile(
    mut multipart: Multipart,
) -> impl IntoResponse {
    let mut fragments: Vec<ChecksumFragment> = Vec::new();

    if let Ok(Some(field)) = multipart.next_field().await {
        let data = field.bytes().await.unwrap();

        let lockfile = toml::from_str::<Lockfile>(
            std::str::from_utf8(&data).unwrap()
        );

        if let Ok(lockfile) = lockfile {
            for package in lockfile.package {
                match package.checksum {
                    Some(checksum) => {
                        match ChecksumFragment::try_from(checksum) {
                            Ok(fragment) => {
                                fragments.push(fragment);
                            },
                            Err(_) => {
                                return Err(StatusCode::UNPROCESSABLE_ENTITY);
                            }
                        }
                    },
                    None => continue,
                }
            }
        } else {
            return Err(StatusCode::BAD_REQUEST);
        }
    } else {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(Html::from(
        fragments.into_iter()
            .map(|fragment| fragment.render().unwrap())
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

#[cfg(test)]
mod test {
    use crate::day23::Lockfile;

    #[test]
    fn test_deserialize_lockfile() {
        let lockfile: &str = include_str!("../Cargo.lock");

        let lockfile = lockfile.lines().skip(2).collect::<Vec<_>>().join("\n");


        assert!(
            matches!(toml::from_str::<Lockfile>(lockfile.as_str()), Ok(_))
        );
    }
}