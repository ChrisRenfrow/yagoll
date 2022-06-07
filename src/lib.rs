use std::{
    fmt::{self, Debug, Display, Formatter},
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

static DEFAULT_BOARD_SIZE: usize = 10;
static DEFAULT_BORDER_BEHAVIOR: BorderOpt = BorderOpt::Solid;

const FILE_LIVE_CHAR: u8 = b'#';
const FILE_DEAD_CHAR: u8 = b'_';

/// Border options
#[derive(Debug, Clone, PartialEq)]
pub enum BorderOpt {
    /// Consider the border as "alive"
    Solid,
    /// Consider the border as "dead"
    Empty,
    /// Consider the border as the opposite side of the board
    Loop,
}

/// A simple cell
#[derive(Debug, Clone, PartialEq)]
pub enum Cell {
    /// The cell is alive (true)
    Alive,
    /// The cell is dead (false)
    Dead,
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        if *self == Cell::Alive {
            write!(f, "▓▓")
        } else {
            write!(f, "░░")
        }
    }
}

/// A Game of Life Board
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    /// The height and width of the board
    pub size: usize,
    /// The cells themselves
    pub cells: Vec<Vec<Cell>>,
    /// The border behavior
    pub border: BorderOpt,
}

impl Board {
    /// Initialize a new board
    pub fn new(size: Option<usize>, border: Option<BorderOpt>) -> Self {
        let size = size.unwrap_or(DEFAULT_BOARD_SIZE);
        let border = border.unwrap_or_else(|| DEFAULT_BORDER_BEHAVIOR.clone());
        Board {
            size,
            border,
            cells: vec![vec![Cell::Dead; size]; size],
        }
    }

    /// Initialize new board from the file at `path`. The file should
    /// only consist of sequences of `#` (alive) and `_` (dead)
    /// characters where each row is delimited by new-lines.
    ///
    /// **Warning:** Assumes the dimensions are square and match the
    /// length of the first line of input
    pub fn new_from_file(path: &Path) -> Self {
        let file = match File::open(&path) {
            Err(why) => panic!("Error opening file{}: {}", path.display(), why),
            Ok(file) => file,
        };
        let mut size = 0;
        let mut cells: Vec<Vec<Cell>> = vec![vec![]];

        BufReader::new(file).lines().enumerate().for_each(|(i, l)| {
            let l = l.unwrap();
            if size == 0 {
                size = l.len();
                cells = vec![vec![Cell::Dead; size]; size];
            }
            cells[i] = Board::parse_str_as_cells(&l);
        });

        Board {
            size,
            cells,
            border: BorderOpt::Empty,
        }
    }

    /// Toggle a cell's state
    pub fn toggle_cell(&mut self, x: usize, y: usize) {
        match (self.is_valid_pos(x, y), self.get_cell(x, y)) {
            (true, Cell::Alive) => self.cells[x][y] = Cell::Dead,
            (true, Cell::Dead) => self.cells[x][y] = Cell::Alive,
            _ => (),
        }
    }

    /// Advance board state by one cycle
    pub fn advance_cycle(&mut self) {
        let mut updates: Vec<(usize, usize, Cell)> = vec![];

        for x in 0..self.size {
            for y in 0..self.size {
                match (self.cell_should_live(x, y), self.get_cell(x, y)) {
                    (true, Cell::Dead) => updates.push((x, y, Cell::Alive)),
                    (false, Cell::Alive) => updates.push((x, y, Cell::Dead)),
                    _ => (),
                }
            }
        }

        for (x, y, cell) in updates {
            self.cells[x][y] = cell;
        }
    }

    /// Advance board state by n cycles
    pub fn advance_n_cycles(&mut self, n: usize) {
        (0..n).for_each(|_| self.advance_cycle())
    }
}

impl Board {
    fn is_valid_pos(&self, x: usize, y: usize) -> bool {
        x < self.size && y < self.size
    }

    fn get_cell(&self, x: usize, y: usize) -> Cell {
        if self.is_valid_pos(x, y) {
            self.cells[x][y].clone()
        } else {
            panic!("{}, {} not valid cell!", x, y);
        }
    }

    fn is_border(&self, x: i32, y: i32) -> bool {
        let board_size = self.size as i32;

        (x < 0 || y < 0) || (x >= board_size || y >= board_size)
    }

    fn get_live_neighbor_count(&self, x: usize, y: usize) -> usize {
        let cell = self.get_cell(x, y);
        let x = x as i32;
        let y = y as i32;
        let mut n = 0;

        (x - 1..x + 2).for_each(|row| {
            (y - 1..y + 2).for_each(|col| {
                if self.is_border(row, col) {
                    match self.border {
                        BorderOpt::Solid => n += 1,
                        BorderOpt::Empty => (),
                        _ => (),
                    }
                } else if self.get_cell(row as usize, col as usize) == Cell::Alive {
                    n += 1;
                }
            });
        });

        if cell == Cell::Alive {
            n - 1
        } else {
            n
        }
    }

    fn cell_should_live(&self, x: usize, y: usize) -> bool {
        let cell = self.get_cell(x, y);

        match self.get_live_neighbor_count(x, y) {
            3 => true,
            2 => cell == Cell::Alive,
            _ => false,
        }
    }

    fn parse_str_as_cells(string: &str) -> Vec<Cell> {
        let mut cells = vec![];

        for c in string.bytes() {
            match c {
                FILE_LIVE_CHAR => cells.push(Cell::Alive),
                FILE_DEAD_CHAR => cells.push(Cell::Dead),
                _ => (),
            }
        }

        cells
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

    // ▓▓░░░░░░
    // ░░▓▓░░░░
    // ░░░░▓▓░░
    // ░░░░░░▓▓
    fn get_4x4_board() -> Board {
        let mut board = Board::new(Some(4), Some(BorderOpt::Empty));

        board.toggle_cell(0, 0);
        board.toggle_cell(1, 1);
        board.toggle_cell(2, 2);
        board.toggle_cell(3, 3);

        board
    }

    // ░░▓▓░░
    // ░░▓▓░░
    // ░░▓▓░░
    fn get_blinker_board() -> Board {
        let mut board = Board::new(Some(3), Some(BorderOpt::Empty));

        board.toggle_cell(0, 1);
        board.toggle_cell(1, 1);
        board.toggle_cell(2, 1);

        board
    }

    // ░░▓▓░░░░░░
    // ░░░░▓▓░░░░
    // ▓▓▓▓▓▓░░░░
    // ░░░░░░░░░░
    // ░░░░░░░░░░
    fn get_glider_board() -> Board {
        let mut board = Board::new(Some(5), Some(BorderOpt::Empty));

        board.toggle_cell(0, 1);
        board.toggle_cell(1, 2);
        board.toggle_cell(2, 0);
        board.toggle_cell(2, 1);
        board.toggle_cell(2, 2);

        board
    }

    fn get_file_board() -> Board {
        Board::new_from_file(Path::new("./test.txt"))
    }

    #[test]
    fn init_default_board() {
        let board = Board::new(None, None);
        assert_eq!(
            board,
            Board {
                size: DEFAULT_BOARD_SIZE,
                border: DEFAULT_BORDER_BEHAVIOR.clone(),
                cells: vec![vec![Cell::Dead; DEFAULT_BOARD_SIZE]; DEFAULT_BOARD_SIZE]
            }
        );
    }

    #[test]
    fn display_board() {
        let board = get_4x4_board();

        assert_eq!(
            format!("{}", board),
            "▓▓░░░░░░\n\
             ░░▓▓░░░░\n\
             ░░░░▓▓░░\n\
             ░░░░░░▓▓\n"
                .to_string()
        );
    }

    #[test]
    fn correct_neighbor_count() {
        let board_4x4 = get_4x4_board();
        let board_blinker = get_blinker_board();

        assert_eq!(board_4x4.get_live_neighbor_count(0, 0), 1);
        assert_eq!(board_4x4.get_live_neighbor_count(1, 1), 2);
        assert_eq!(board_4x4.get_live_neighbor_count(3, 3), 1);

        assert_eq!(board_blinker.get_live_neighbor_count(0, 0), 2);
        assert_eq!(board_blinker.get_live_neighbor_count(1, 1), 2);
        assert_eq!(board_blinker.get_live_neighbor_count(2, 1), 1);
    }

    #[test]
    fn should_cell_live() {
        let board = get_4x4_board();

        assert!(!board.cell_should_live(0, 0));
        assert!(board.cell_should_live(1, 1));
        assert!(!board.cell_should_live(3, 3));
    }

    #[test]
    fn blinker_should_blink() {
        let mut board = get_blinker_board();

        println!("{}", board);
        board.advance_cycle();
        println!("{}", board);

        assert_eq!(
            format!("{}", board),
            "░░░░░░\n\
             ▓▓▓▓▓▓\n\
             ░░░░░░\n"
                .to_string()
        );

        board.advance_cycle();
        println!("{}", board);

        assert_eq!(
            format!("{}", board),
            "░░▓▓░░\n\
             ░░▓▓░░\n\
             ░░▓▓░░\n"
                .to_string()
        );
    }

    #[test]
    fn gilder_should_glide() {
        let mut board = get_glider_board();
        let expected = "\
        ░░░░░░░░░░\n\
        ░░░░░░░░░░\n\
        ░░░░░░▓▓░░\n\
        ░░░░░░░░▓▓\n\
        ░░░░▓▓▓▓▓▓\n";

        board.advance_n_cycles(8); // 8 cycles to fully traverse board

        println!("Expected:\n{}\nActual:\n{}", expected, board);
        assert_eq!(format!("{}", board), expected.to_string());
    }

    #[test]
    fn file_should_file() {
        let board = get_file_board();
        let expected = "\
        ░░░░░░░░░░\n\
        ░░░░░░░░░░\n\
        ░░░░░░▓▓░░\n\
        ░░░░░░░░▓▓\n\
        ░░░░▓▓▓▓▓▓\n";

        println!("Expected:\n{}\nActual:\n{}", expected, board);
        assert_eq!(format!("{}", board), expected.to_string());
    }
}
