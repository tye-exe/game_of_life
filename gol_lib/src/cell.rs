/// Represents the state of a cell within the Conways game of life simulation.
///
/// An alive cell is represented as `true`.
/// A dead cell is represented as `false`.
#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Cell {
    Alive,
    Dead,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Dead
    }
}

impl From<Cell> for bool {
    fn from(value: Cell) -> Self {
        match value {
            Cell::Alive => true,
            Cell::Dead => false,
        }
    }
}

impl From<bool> for Cell {
    fn from(value: bool) -> Self {
        match value {
            true => Cell::Alive,
            false => Cell::Dead,
        }
    }
}

impl Cell {
    /// Returns the opposite of the current cell.
    pub fn invert(self) -> Cell {
        match self {
            Cell::Alive => Cell::Dead,
            Cell::Dead => Cell::Alive,
        }
    }
}
