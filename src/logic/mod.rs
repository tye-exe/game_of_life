use std::collections::HashMap;

/// Contains two board, one which the UI will read from to display, the other will be used to write
/// the result of the current simulation step to.
pub struct Board {
    current_board: Box<HashMap<Position, Cell>>,
    buffer_board: Box<HashMap<Position, Cell>>,
}

/// The x & y positions of a cell on the board.
pub struct Position {
    x: u16,
    y: u16,
}

impl Position {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

/// A cell on the board.
#[derive(Debug, PartialEq)]
pub enum Cell {
    Alive,
    Dead,
}

impl Board {
    pub fn set(&self, position: Position, cell: Cell) {
        todo!()
    }

    pub fn get(&self, position: Position) -> Cell {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Default for Board {
        fn default() -> Self {
            todo!()
        }
    }

    #[test]
    /// Sets a cell to be alive
    fn set_cell_alive() {
        let board = Board::default();
        board.set(Position::new(0, 0), Cell::Alive);
        let cell = board.get(Position::new(0, 0));
        assert_eq!(cell, Cell::Alive);
    }
}
