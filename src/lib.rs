use std::{
    fmt::{self, Debug, Display, Formatter},
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

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
#[derive(Debug, Clone, Copy, PartialEq)]
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
pub struct Board<const WIDTH: usize, const HEIGHT: usize> {
    /// 2-dimensional array of Cells
    pub cells: [[Cell; WIDTH]; HEIGHT],
    /// The border behavior
    pub border: BorderOpt,
}

impl<const WIDTH: usize, const HEIGHT: usize> Board<WIDTH, HEIGHT> {
    /// Initialize a new board
    ///
    /// # Usage:
    ///
    /// `Board::<X, Y>::new(BorderOpt::Empty)`
    pub fn new(border: BorderOpt) -> Self {
        Board {
            border,
            cells: [[Cell::Dead; WIDTH]; HEIGHT],
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
    /// - If the length of a line doesn't match `WIDTH`
    /// - If the number of lines exceeds `HEIGHT`
    pub fn new_from_file(path: &Path) -> Self {
        let file = match File::open(&path) {
            Err(why) => panic!("Error opening file{}: {}", path.display(), why),
            Ok(file) => file,
        };
        let mut cells = [[Cell::Dead; WIDTH]; HEIGHT];
        let mut line_iter = BufReader::new(file).lines();
        let border_str = line_iter.next().unwrap().unwrap();
        let border = Self::parse_str_as_border_opt(&border_str).unwrap_or(BorderOpt::Empty);

        line_iter.enumerate().for_each(|(i, l)| {
            let l = l.unwrap();
            let l = l.trim();
            if WIDTH != l.len() {
                panic!("width of line {} is {}, expected {}", i + 1, l.len(), WIDTH);
            }

            cells[i] = Self::parse_str_as_cells(l);
        });

        Board { cells, border }
    }

    /// Advance board state by one cycle
    pub fn advance_cycle(&mut self) {
        let mut updates: Vec<(usize, usize, Cell)> = vec![];

        (0..WIDTH).for_each(|x| {
            (0..HEIGHT).for_each(|y| match (self.cell_should_live(x, y), self.get(x, y)) {
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
        self.cells[x][y] = c;
    }

    /// Get cell at `x` and `y`
    ///
    /// # Panics:
    ///
    /// If `x` or `y` are out of range
    pub fn get(&self, x: usize, y: usize) -> Cell {
        self.cells[x][y]
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Board<WIDTH, HEIGHT> {
    fn is_border(&self, x: i32, y: i32) -> bool {
        (x < 0 || y < 0) || (x >= WIDTH as i32 || y >= HEIGHT as i32)
    }

    fn get_live_neighbor_count(&self, x: usize, y: usize) -> usize {
        let cell = self.get(x, y);
        let x = x as i32;
        let y = y as i32;

        let mut n = 0;

        // TODO: Refactor this out into another method which retrieves a slice of neighbors
        (x - 1..x + 2).for_each(|x| {
            (y - 1..y + 2).for_each(|y| {
                n += if self.is_border(x, y) {
                    match self.border {
                        BorderOpt::Solid => 1,
                        BorderOpt::Empty => 0,
                        _ => 0,
                    }
                } else if self.get(x as usize, y as usize) == Cell::Alive {
                    1
                } else {
                    0
                };
            });
        });

        if cell == Cell::Alive {
            n - 1
        } else {
            n
        }
    }

    fn cell_should_live(&self, x: usize, y: usize) -> bool {
        let cell = self.get(x, y);

        match self.get_live_neighbor_count(x, y) {
            3 => true,
            2 => cell == Cell::Alive,
            _ => false,
        }
    }

    fn parse_str_as_cells(string: &str) -> [Cell; WIDTH] {
        let mut cell_row = [Cell::Dead; WIDTH];

        string.bytes().enumerate().for_each(|(i, c)| {
            cell_row[i] = match c {
                FILE_LIVE_CHAR => Cell::Alive,
                FILE_DEAD_CHAR => Cell::Dead,
                _ => Cell::Dead,
            }
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

impl<const WIDTH: usize, const HEIGHT: usize> Default for Board<WIDTH, HEIGHT> {
    fn default() -> Self {
        Board::<WIDTH, HEIGHT>::new(BorderOpt::Empty)
    }
}

impl<const WIDTH: usize, const HEIGHT: usize> Display for Board<WIDTH, HEIGHT> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        self.cells.iter().for_each(|x| {
            x.iter().for_each(|c| {
                write!(f, "{}", c).unwrap();
            });
            writeln!(f).unwrap();
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
    fn get_4x4_board() -> Board<4, 4> {
        let mut board = Board::<4, 4>::new(BorderOpt::Empty);

        board.set(0, 0, Cell::Alive);
        board.set(1, 1, Cell::Alive);
        board.set(2, 2, Cell::Alive);
        board.set(3, 3, Cell::Alive);

        board
    }

    // ░░▓▓░░
    // ░░▓▓░░
    // ░░▓▓░░
    fn get_blinker_board() -> Board<3, 3> {
        let mut board = Board::<3, 3>::new(BorderOpt::Empty);

        board.set(0, 1, Cell::Alive);
        board.set(1, 1, Cell::Alive);
        board.set(2, 1, Cell::Alive);

        board
    }

    // ░░▓▓░░░░░░
    // ░░░░▓▓░░░░
    // ▓▓▓▓▓▓░░░░
    // ░░░░░░░░░░
    // ░░░░░░░░░░
    fn get_glider_board() -> Board<5, 5> {
        let mut board = Board::<5, 5>::new(BorderOpt::Empty);

        board.set(0, 1, Cell::Alive);
        board.set(1, 2, Cell::Alive);
        board.set(2, 0, Cell::Alive);
        board.set(2, 1, Cell::Alive);
        board.set(2, 2, Cell::Alive);

        board
    }

    fn get_rectangular_board() -> Board<5, 3> {
        let mut board = Board::<5, 3>::new(BorderOpt::Empty);

        board.set(1, 1, Cell::Alive);
        board.set(1, 2, Cell::Alive);
        board.set(1, 3, Cell::Alive);

        board
    }

    fn get_file_board() -> Board<5, 5> {
        Board::new_from_file(Path::new("./test.txt"))
    }

    fn get_bad_file_board() -> Board<5, 3> {
        Board::new_from_file(Path::new("./bad-test.txt"))
    }

    #[test]
    fn init_default_board() {
        let board = Board::<10, 10>::default();
        assert_eq!(
            board,
            Board::<10, 10> {
                border: BorderOpt::Empty,
                cells: [[Cell::Dead; 10]; 10]
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
        ░░░░░░░░░░\n\
        ░░░░░░▓▓░░\n\
        ░░░░░░░░▓▓\n\
        ░░░░▓▓▓▓▓▓\n";

        println!("{:?}", board);

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
