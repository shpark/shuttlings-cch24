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

pub(super) struct Board<const N: usize> {
    tiles: Vec<Vec<Tile>>,
}

impl<const N: usize> Board<N> {
    pub(super) fn new() -> Self {
        Board {
            tiles: iter::repeat_n(
                iter::repeat_n(Tile::default(), N).collect::<Vec<_>>(), N
            )
            .collect::<Vec<_>>(),
        }
    }

    fn reset(&mut self) {
        for row in 0..self.tiles.len() {
            for col in 0..self.tiles[0].len() {
                self.tiles[row][col] = Tile::default();
            }
        }
    }
}

impl<const N: usize> fmt::Display for Board<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..N {
            write!(f, "{}", WALL)?;

            for col in 0..N {
                write!(f, "{}", self.tiles[row][col])?;
            }

            writeln!(f, "{}", WALL)?;
        }

        for _ in 0..(N + 2) {
            write!(f, "{}", WALL)?;
        }

        writeln!(f, "")?;

        Ok(())
    }
}

pub(super) async fn board(
    State(state): State<AppState>,
) -> impl IntoResponse {
    (StatusCode::OK, state.board.read().await.to_string())
}

pub(super) async fn reset(
    State(state): State<AppState>,
) -> impl IntoResponse {
    state.board.write().await.reset();

    /* Race condition? */

    (StatusCode::OK, state.board.read().await.to_string())
}

#[cfg(test)]
mod test {
    use super::*;

const EMPTY_BOARD: &str = "\
⬜⬛⬛⬛⬛⬜
⬜⬛⬛⬛⬛⬜
⬜⬛⬛⬛⬛⬜
⬜⬛⬛⬛⬛⬜
⬜⬜⬜⬜⬜⬜
";

    #[test]
    fn test_fmt() {
        let board: Board<4> = Board::new();

        assert_eq!(
            format!("{}", board),
            String::from(EMPTY_BOARD),
        )
    }
}
