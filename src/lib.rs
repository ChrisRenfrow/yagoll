/* Rules:
 * - Any live cell with two or three live neighbours survives.
 * - Any dead cell with three live neighbours becomes a live cell.
 * - All other live cells die in the next generation. Similarly, all other dead cells stay dead.
 */

use std::{
    fmt::{self, Debug, Display, Formatter},
    io::Error,
};

static DEFAULT_BOARD_SIZE: usize = 10;
static DEFAULT_BORDER_BEHAVIOR: BorderOpt = BorderOpt::Solid;

#[derive(Debug, Clone, PartialEq)]
enum BorderOpt {
    Solid,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CellState {
    Alive,
    Dead,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Cell {
    state: CellState,
}

impl Cell {
    fn new(state: Option<CellState>) -> Self {
        Cell {
            state: state.unwrap_or(CellState::Dead),
        }
    }
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if self.state == CellState::Alive {
            write!(f, "▓▓")
        } else {
            write!(f, "░░")
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    size: usize,
    cells: Vec<Vec<Cell>>,
    border: BorderOpt,
}

impl Board {
    fn new(size: Option<usize>, border: Option<BorderOpt>) -> Self {
        let size = size.unwrap_or(DEFAULT_BOARD_SIZE);
        let border = border.unwrap_or_else(|| DEFAULT_BORDER_BEHAVIOR.clone());
        Board {
            size,
            border,
            cells: vec![vec![Cell::new(None); size]; size],
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let cells = &self.cells;
        for r in cells {
            for c in r {
                write!(f, "{}", c)?;
            }
            writeln!(f)?;
        }
        write!(f, "")
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn init_default_cell() {
        let cell = Cell::new(None);
        assert_eq!(
            cell,
            Cell {
                state: CellState::Dead
            }
        );
    }

    #[test]
    fn init_alive_cell() {
        let cell = Cell::new(Some(CellState::Alive));
        assert_eq!(
            cell,
            Cell {
                state: CellState::Alive,
            }
        );
    }

    #[test]
    fn init_default_board() {
        let board = Board::new(None, None);
        assert_eq!(
            board,
            Board {
                size: DEFAULT_BOARD_SIZE,
                border: DEFAULT_BORDER_BEHAVIOR.clone(),
                cells: vec![vec![Cell::new(None); DEFAULT_BOARD_SIZE]; DEFAULT_BOARD_SIZE]
            }
        );
    }

    #[test]
    fn display_board() {
        let mut board = Board::new(Some(4), Some(BorderOpt::Solid));

        board.cells[0][0].state = CellState::Alive;
        board.cells[1][1].state = CellState::Alive;
        board.cells[2][2].state = CellState::Alive;
        board.cells[3][3].state = CellState::Alive;

        assert_eq!(
            format!("{}", board),
            "▓▓░░░░░░\n\
						 ░░▓▓░░░░\n\
						 ░░░░▓▓░░\n\
						 ░░░░░░▓▓\n"
                .to_string()
        );
    }
}
