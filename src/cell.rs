use std::fmt::{Display, Formatter, Result};

/// A simple cell
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    /// The cell is alive (true)
    Alive,
    /// The cell is dead (false)
    Dead,
}

impl Display for Cell {
    fn fmt(&self, f: &mut Formatter) -> Result {
        if *self == Cell::Alive {
            write!(f, "▓▓")
        } else {
            write!(f, "░░")
        }
    }
}
