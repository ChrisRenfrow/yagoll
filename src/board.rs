use std::{
    fmt::{self, Debug, Display, Formatter},
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::cell::Cell;

const FILE_LIVE_CHAR: u8 = b'#';
const FILE_DEAD_CHAR: u8 = b'_';

/// Border options
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BorderOpt {
    /// Consider the border as "alive"
    Solid,
    /// Consider the border as "dead"
    Empty,
    /// Consider the border as the opposite side of the board
    Loop,
}

/// A Game of Life Board
#[derive(Debug, Clone, PartialEq)]
pub struct Board {
    /// The width of the board
    pub width: usize,
    /// The height of the board
    pub height: usize,
    /// The border behavior
    pub border: BorderOpt,
    /// Private array of Cells
    cells: Vec<Cell>,
}

impl Board {
    /// Initialize a new board
    ///
    /// # Example:
    ///
    /// ```
    /// use yagoll::*;
    ///
    /// let mut board = Board::new(5, 5, BorderOpt::Empty);
    /// assert!(board.width == 5 && board.height == 5 && board.border == BorderOpt::Empty);
    /// assert_eq!(board.get(2,2), Cell::Dead);
    /// ```
    pub fn new(width: usize, height: usize, border: BorderOpt) -> Self {
        Board {
            width,
            height,
            border,
            cells: vec![Cell::Dead; width * height],
        }
    }

    /// Initialize new board from the file at `path`.
    ///
    /// # File format:
    ///
    /// The file should start with one of the
    /// following border options:
    ///
    /// - `empty`
    /// - `solid`
    ///
    /// Followed by lines consisting of `#` (alive) and `_` (dead)
    /// characters.
    ///
    /// # Panics:
    ///
    /// - If the file is invalid or non-existent
    /// - If the length of a line exceeds the width of the first line
    ///
    /// # Example:
    /// ```
    /// use yagoll::*;
    ///
    /// let board = Board::new_from_file("./tests/test-boards/glider.txt");
    ///
    /// assert!(board.width == 5 && board.height == 5 && board.border == BorderOpt::Empty);
    /// assert_eq!(board.get(2, 1), Cell::Alive);
    /// ```
    pub fn new_from_file(path: &str) -> Self {
        let file = match File::open(Path::new(path)) {
            Err(why) => panic!("Error opening file{}: {}", path, why),
            Ok(file) => file,
        };
        let mut cells: Vec<Cell> = vec![];
        let mut line_iter = BufReader::new(file).lines();
        let (mut width, mut height) = (0, 0);
        let border_str = line_iter.next().unwrap().unwrap();
        let border = Self::parse_str_as_border_opt(&border_str).unwrap_or(BorderOpt::Empty);

        line_iter.enumerate().for_each(|(_i, l)| {
            let l = l.unwrap();
            let l = l.trim();
            width = if width == 0 { l.len() } else { width };
            if l.len() != width {
                panic!("row {} is length {}, expected {}", _i, l.len(), width);
            }
            cells.append(&mut Self::parse_str_as_cells(l));
            height += 1;
        });

        Board {
            width,
            height,
            cells,
            border,
        }
    }

    /// Advance board state by one cycle
    pub fn advance_cycle(&mut self) {
        let mut updates: Vec<(usize, usize, Cell)> = vec![];

        (0..self.width).for_each(|x| {
            (0..self.height).for_each(|y| match (self.cell_should_live(x, y), self.get(x, y)) {
                (true, Cell::Dead) => updates.push((x, y, Cell::Alive)),
                (false, Cell::Alive) => updates.push((x, y, Cell::Dead)),
                _ => (),
            })
        });

        updates.iter().for_each(|&(x, y, cell)| {
            self.set(x, y, cell);
        });
    }

    /// Advance board state by n cycles
    pub fn advance_n_cycles(&mut self, n: usize) {
        (0..n).for_each(|_| self.advance_cycle())
    }

    /// Set cell at `x` and `y` to state `c`
    ///
    /// # Panics:
    ///
    /// If `x` or `y` are out of range
    pub fn set(&mut self, x: usize, y: usize, c: Cell) {
        let idx = self.to_idx(x, y);
        self.cells[idx] = c;
    }

    /// Get cell at `x` and `y`
    ///
    /// # Panics:
    ///
    /// If `x` or `y` are out of range
    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[self.to_idx(x, y)]
    }
}

impl Board {
    fn to_idx(&self, x: usize, y: usize) -> usize {
        if x >= self.width {
            panic!("out of bounds: width is {} but x is {}", self.width, x);
        } else if y >= self.height {
            panic!("out of bounds: height is {} but y is {}", self.height, y);
        }
        ((y % self.height) * self.width) + x
    }

    // ___
    // _X#
    // _##
    fn get_lower_right_neighbors(&self, x: usize, y: usize) -> Vec<Cell> {
        [
            &[self.get(x + 1, y)][..],
            &[self.get(x, y + 1), self.get(x + 1, y + 1)][..],
        ]
        .concat()
        .to_vec()
    }

    // ###
    // #X#
    // ___
    fn get_upper_neighbors(&self, x: usize, y: usize) -> Vec<Cell> {
        [
            &self.cells[self.to_idx(x - 1, y - 1)..self.to_idx(x + 1, y - 1) + 1],
            &[self.get(x - 1, y), self.get(x + 1, y)][..],
        ]
        .concat()
        .to_vec()
    }

    // ___
    // #X#
    // ###
    fn get_lower_neighbors(&self, x: usize, y: usize) -> Vec<Cell> {
        [
            &[self.get(x - 1, y), self.get(x + 1, y)][..],
            &self.cells[self.to_idx(x - 1, y + 1)..self.to_idx(x + 1, y + 1) + 1],
        ]
        .concat()
        .to_vec()
    }

    // _##
    // _X#
    // _##
    fn get_right_neighbors(&self, x: usize, y: usize) -> Vec<Cell> {
        [
            &self.cells[self.to_idx(x, y - 1)..self.to_idx(x + 1, y - 1) + 1],
            &[self.get(x + 1, y)][..],
            &self.cells[self.to_idx(x, y + 1)..self.to_idx(x + 1, y + 1) + 1],
        ]
        .concat()
        .to_vec()
    }

    // _##
    // _X#
    // ___
    fn get_upper_right_neighbors(&self, x: usize, y: usize) -> Vec<Cell> {
        [
            &[self.get(x, y - 1), self.get(x + 1, y - 1)][..],
            &[self.get(x + 1, y)][..],
        ]
        .concat()
        .to_vec()
    }

    // ___
    // #X_
    // ##_
    fn get_lower_left_neighbors(&self, x: usize, y: usize) -> Vec<Cell> {
        [
            &[self.get(x - 1, y)][..],
            &[self.get(x - 1, y + 1), self.get(x, y + 1)][..],
        ]
        .concat()
        .to_vec()
    }

    // ##_
    // #X_
    // ##_
    fn get_left_neighbors(&self, x: usize, y: usize) -> Vec<Cell> {
        [
            &self.cells[self.to_idx(x - 1, y - 1)..self.to_idx(x, y - 1) + 1],
            &[self.get(x - 1, y)][..],
            &self.cells[self.to_idx(x - 1, y + 1)..self.to_idx(x, y + 1) + 1],
        ]
        .concat()
        .to_vec()
    }

    // ##_
    // #X_
    // ___
    fn get_upper_left_neighbors(&self, x: usize, y: usize) -> Vec<Cell> {
        [
            &[self.get(x - 1, y - 1), self.get(x, y - 1)][..],
            &[self.get(x - 1, y)][..],
        ]
        .concat()
        .to_vec()
    }

    // ###
    // #X#
    // ###
    fn get_all_neighbors(&self, x: usize, y: usize) -> Vec<Cell> {
        [
            &self.cells[self.to_idx(x - 1, y - 1)..self.to_idx(x + 1, y - 1) + 1],
            &[self.get(x - 1, y), self.get(x + 1, y)][..],
            &self.cells[self.to_idx(x - 1, y + 1)..self.to_idx(x + 1, y + 1) + 1],
        ]
        .concat()
        .to_vec()
    }

    fn get_neighbors(&self, x: usize, y: usize) -> Vec<Cell> {
        let w: usize = self.width - 1;
        let h: usize = self.height - 1;

        match (x, y) {
            (x, y) if (x > 0 && x < w) && y == h => self.get_upper_neighbors(x, y),
            (0, y) if y == h => self.get_upper_right_neighbors(x, y),
            (0, y) if y > 0 && y < h => self.get_right_neighbors(x, y),
            (0, 0) => self.get_lower_right_neighbors(x, y),
            (x, 0) if x > 0 && x < w => self.get_lower_neighbors(x, y),
            (x, 0) if x == w => self.get_lower_left_neighbors(x, y),
            (x, y) if x == w && (y > 0 && y < h) => self.get_left_neighbors(x, y),
            (x, y) if x == w && y == h => self.get_upper_left_neighbors(x, y),
            _ => self.get_all_neighbors(x, y),
        }
    }

    fn get_live_neighbor_count(&self, x: usize, y: usize) -> usize {
        self.get_neighbors(x, y)
            .iter()
            .filter(|n| **n == Cell::Alive)
            .count()
    }

    fn cell_should_live(&self, x: usize, y: usize) -> bool {
        let cell = self.get(x, y);

        match self.get_live_neighbor_count(x, y) {
            3 => true,
            2 => cell == Cell::Alive,
            _ => false,
        }
    }

    fn parse_str_as_cells(string: &str) -> Vec<Cell> {
        let mut cell_row: Vec<Cell> = vec![];

        string.bytes().for_each(|c| {
            cell_row.push(match c {
                FILE_LIVE_CHAR => Cell::Alive,
                FILE_DEAD_CHAR => Cell::Dead,
                _ => Cell::Dead,
            })
        });

        cell_row
    }

    fn parse_str_as_border_opt(string: &str) -> Option<BorderOpt> {
        match string {
            "solid" => Some(BorderOpt::Solid),
            "empty" => Some(BorderOpt::Empty),
            _ => None,
        }
    }
}

impl Default for Board {
    fn default() -> Self {
        Board::new(10, 10, BorderOpt::Empty)
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.cells.iter().enumerate().for_each(|(i, c)| {
            if (i + 1) % self.width == 0 {
                writeln!(f, "{}", c).unwrap()
            } else {
                write!(f, "{}", c).unwrap()
            }
        });
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
        let mut board = Board::new(4, 4, BorderOpt::Empty);

        board.set(0, 0, Cell::Alive);
        board.set(1, 1, Cell::Alive);
        board.set(2, 2, Cell::Alive);
        board.set(3, 3, Cell::Alive);

        board
    }

    // ░░▓▓░░
    // ░░▓▓░░
    // ░░▓▓░░
    fn get_blinker_board() -> Board {
        let mut board = Board::new(3, 3, BorderOpt::Empty);

        board.set(1, 0, Cell::Alive);
        board.set(1, 1, Cell::Alive);
        board.set(1, 2, Cell::Alive);

        board
    }

    // ░░▓▓░░░░░░
    // ░░░░▓▓░░░░
    // ▓▓▓▓▓▓░░░░
    // ░░░░░░░░░░
    // ░░░░░░░░░░
    fn get_glider_board() -> Board {
        let mut board = Board::new(5, 5, BorderOpt::Empty);

        board.set(0, 1, Cell::Alive);
        board.set(1, 2, Cell::Alive);
        board.set(2, 0, Cell::Alive);
        board.set(2, 1, Cell::Alive);
        board.set(2, 2, Cell::Alive);

        board
    }

    fn get_rectangular_board() -> Board {
        let mut board = Board::new(5, 3, BorderOpt::Empty);

        board.set(1, 1, Cell::Alive);
        board.set(2, 1, Cell::Alive);
        board.set(3, 1, Cell::Alive);

        board
    }

    fn get_file_board() -> Board {
        Board::new_from_file("./tests/test-boards/glider.txt")
    }

    fn get_bad_file_board() -> Board {
        Board::new_from_file("./tests/test-boards/bad-test.txt")
    }

    #[test]
    fn init_default_board() {
        let board = Board::default();
        assert_eq!(
            board,
            Board {
                width: 10,
                height: 10,
                border: BorderOpt::Empty,
                cells: vec![Cell::Dead; 10 * 10],
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
        assert_eq!(board_blinker.get_live_neighbor_count(2, 1), 3);
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
        ░░░░░░▓▓░░\n\
        ░░░░░░░░▓▓\n\
        ░░░░▓▓▓▓▓▓\n\
        ░░░░░░░░░░\n";

        board.advance_n_cycles(6); // 8 cycles to fully traverse board

        println!("Expected:\n{}\nActual:\n{}", expected, board);
        assert_eq!(format!("{}", board), expected.to_string());
    }

    #[test]
    fn rectangle_should_rectangle() {
        let board = get_rectangular_board();
        let expected = "\
        ░░░░░░░░░░\n\
        ░░▓▓▓▓▓▓░░\n\
        ░░░░░░░░░░\n";

        println!("{:#?}", board);

        println!("Expected:\n{}\nActual:\n{}", expected, board);
        assert_eq!(format!("{}", board), expected.to_string());
    }

    #[test]
    fn file_should_file() {
        let board = get_file_board();
        let expected = "\
        ░░░░░░░░░░\n\
        ░░░░▓▓░░░░\n\
        ░░░░░░▓▓░░\n\
        ░░▓▓▓▓▓▓░░\n\
        ░░░░░░░░░░\n";

        println!("{:#?}", board);

        assert!(board.height == 5 && board.width == 5);
        assert!(board.border == BorderOpt::Empty);

        println!("Expected:\n{}\nActual:\n{}", expected, board);
        assert_eq!(format!("{}", board), expected.to_string());
    }

    #[test]
    #[should_panic]
    fn bad_file() {
        get_bad_file_board();
    }
}
