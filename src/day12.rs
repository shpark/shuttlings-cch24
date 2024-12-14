use std::{fmt, iter};

use axum::{extract::State, http::StatusCode, response::IntoResponse};

use crate::AppState;

const WALL: char = '⬜';

#[derive(Clone, Copy)]
pub(super) enum Tile {
    Empty,
    Cookie,
    Milk,
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Empty
    }
}

impl Into<char> for &Tile {
    fn into(self) -> char {
        match self {
            Tile::Empty => '⬛',
            Tile::Cookie=> '🍪',
            Tile::Milk=> '🥛',
        }
    }
}

impl fmt::Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c: char = self.into();
        write!(f, "{}", c)
    }
}

pub(super) struct Board<const ROWS: usize, const COLS: usize> {
    tiles: Vec<Vec<Tile>>,
}

impl<const ROWS: usize, const COLS: usize> Board<ROWS, COLS> {
    pub(super) fn new() -> Self {
        Board {
            tiles: iter::repeat_n(
                iter::repeat_n(Tile::default(), COLS).collect::<Vec<_>>(),
                ROWS
            )
            .collect::<Vec<_>>(),
        }
    }
}

impl<const ROWS: usize, const COLS: usize> fmt::Display for Board<ROWS, COLS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..ROWS {
            write!(f, "{}", WALL)?;

            for col in 0..COLS {
                write!(f, "{}", self.tiles[row][col])?;
            }

            writeln!(f, "{}", WALL)?;
        }

        for _ in 0..(COLS + 2) {
            write!(f, "{}", WALL)?;
        }

        Ok(())
    }
}

pub(super) async fn board(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    StatusCode::OK
}

pub(super) async fn reset(
    State(_state): State<AppState>,
) -> impl IntoResponse {
    StatusCode::OK
}

#[cfg(test)]
mod test {
    use super::*;

const EMPTY_BOARD: &str = r#"⬜⬛⬛⬛⬛⬜
⬜⬛⬛⬛⬛⬜
⬜⬛⬛⬛⬛⬜
⬜⬛⬛⬛⬛⬜
⬜⬜⬜⬜⬜⬜"#;

    #[test]
    fn test_fmt() {
        let board: Board<4, 4> = Board::new();

        assert_eq!(
            format!("{}", board),
            String::from(EMPTY_BOARD),
        )
    }
}
