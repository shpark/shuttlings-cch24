use askama::Template;
use axum::{extract::Path, http::StatusCode, response::{Html, IntoResponse}};

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
