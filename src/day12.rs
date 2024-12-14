use std::{fmt, iter};

use axum::{extract::{Path, State}, http::StatusCode, response::IntoResponse};

use crate::AppState;

const COOKIE: &str = "cookie";

const MILK: &str = "milk";

const WALL: char = '⬜';

const N: usize = 4;

const NO_WINNER: &str = "No winner.\n";

const WINS: &str = " wins!\n";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(super) enum Team {
    Empty,
    Cookie,
    Milk,
}

impl Default for Team {
    fn default() -> Self {
        Team::Empty
    }
}

impl Into<char> for &Team {
    fn into(self) -> char {
        match self {
            Team::Empty => '⬛',
            Team::Cookie=> '🍪',
            Team::Milk=> '🥛',
        }
    }
}

impl fmt::Display for Team {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let c: char = self.into();
        write!(f, "{}", c)
    }
}

pub(super) struct Board<const N: usize> {
    tiles: Vec<Vec<Team>>,
}

impl<const N: usize> Board<N> {
    pub(super) fn new() -> Self {
        Board {
            tiles: iter::repeat(
                iter::repeat(Team::default()).take(N).collect::<Vec<_>>()
            )
            .take(N)
            .collect::<Vec<_>>(),
        }
    }

    fn reset(&mut self) {
        for row in 0..self.tiles.len() {
            for col in 0..self.tiles[0].len() {
                self.tiles[row][col] = Team::default();
            }
        }
    }

    // N=4 for example:
    //
    // col 1             col 4
    // vvvvv             vvvvv
    // (0,0) (0,1) (0,2) (0,3)
    // (1,0) (1,1) (1,2) (1,3)
    // (2,0) (2,1) (2,2) (2,3)
    // (3,0) (3,1) (3,2) (3,3)
    fn place(&mut self, col: usize, tile: Team) -> Result<(), &'static str> {
        if col < 1 || col > N {
            return Err("meheh");
        }

        for i in (0..N).rev() {
            match self.tiles[i][col - 1] {
                Team::Empty => {
                    self.tiles[i][col - 1] = tile;
                    return Ok(())
                },
                _ => {},
            }
        }

        Err("meh")
    }

    // Returns Some(winner) if a winner exists. This function never returns
    // Some(Team::default()).
    fn winner(&self) -> Option<Team> {
        // Scan rows
        for i in 0..N {
            let maybe_winner = (0..N).fold(self.tiles[i][0], |acc, j| {
                if acc == self.tiles[i][j] { acc } else { Team::default() }
            });

            if matches!(maybe_winner, Team::Cookie | Team::Milk) {
                return Some(maybe_winner);
            }
        }

        // Scan cols
        for i in 0..N {
            let maybe_winner = (0..N).fold(self.tiles[0][i], |acc, j| {
                if acc == self.tiles[j][i] { acc } else { Team::default() }
            });

            if matches!(maybe_winner, Team::Cookie | Team::Milk) {
                return Some(maybe_winner);
            }
        }

        // Scan diagonal lines
        let maybe_winner = (0..N).fold(self.tiles[0][0], |acc, i| {
            if acc == self.tiles[i][i] { acc } else { Team::default() }
        });

        if matches!(maybe_winner, Team::Cookie | Team::Milk) {
            return Some(maybe_winner);
        }

        let maybe_winner = (0..N).fold(self.tiles[0][N - 1], |acc, i| {
            if acc == self.tiles[i][N - 1 - i] { acc } else { Team::default() }
        });

        if matches!(maybe_winner, Team::Cookie | Team::Milk) {
            return Some(maybe_winner);
        }

        None
    }

    fn full(&self) -> bool {
        self.tiles.iter().all(|col| col.iter().all(|&t| t != Team::default()))
    }

    fn winner_alert(&self) -> String {
        if let Some(t) = self.winner() {
            format!("{}{}", t, WINS)
        } else if self.full() {
            format!("{}", NO_WINNER)
        } else {
            String::new()
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
    let board = state.board.read().await;

    (
        StatusCode::OK,
        format!(
            "{}{}",
            board,
            board.winner_alert()
        ),
    )
}

pub(super) async fn reset(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut board = state.board.write().await;

    board.reset();

    (StatusCode::OK, board.to_string())
}

pub(super) async fn place(
    Path((team, column)): Path<(String, usize)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    // {team} is either cookie or milk. {column} is a number between 1 and 4.
    // If either is invalid, return 400 Bad Request (response body does not
    // matter).
    if column < 1 || column > N {
        return (StatusCode::BAD_REQUEST, String::new());
    }

    let tile = if &team == COOKIE {
        Team::Cookie
    } else if team == MILK {
        Team::Milk
    } else {
        return (StatusCode::BAD_REQUEST, String::new());
    };

    let mut board = state.board.write().await;

    // If the game is over (has a winner or no winner), return the board
    // with a 503 Service Unavailable status.
    if let Some(t) = board.winner() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("{}{}{}", board, t, WINS)
        );
    } else if board.full() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("{}{}", board, NO_WINNER),
        );
    }

    // The endpoint should place the incoming item by letting it fall down the
    // column and land in the lowest empty tile. After the new item has been
    // placed, return the board with a 200 OK status.
    if let Ok(_) = board.place(column, tile) {
        let msg = board.winner_alert();

        (StatusCode::OK, format!("{}{}", board, msg))
    } else {
        // If the column requested is already full, return the board with
        // a 503 Service Unavailable status.
        (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("{}", board),
        )
    }
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

    #[test]
    fn test_mini() {
        let mut board: Board<4> = Board::new();

        let _ = board.place(1, Team::Cookie);
        let _ = board.place(2, Team::Milk);
        let _ = board.place(2, Team::Cookie);
        let _ = board.place(3, Team::Milk);
        let _ = board.place(3, Team::Milk);
        let _ = board.place(3, Team::Cookie);
        let _ = board.place(4, Team::Milk);
        let _ = board.place(4, Team::Milk);
        let _ = board.place(4, Team::Milk);
        let _ = board.place(4, Team::Cookie);

        assert_eq!(
            board.to_string(),
            String::from("\
⬜⬛⬛⬛🍪⬜
⬜⬛⬛🍪🥛⬜
⬜⬛🍪🥛🥛⬜
⬜🍪🥛🥛🥛⬜
⬜⬜⬜⬜⬜⬜
"
        ));

        assert_eq!(board.winner(), Some(Team::Cookie));
    }
}
