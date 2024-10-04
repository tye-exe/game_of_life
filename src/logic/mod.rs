use std::collections::HashSet;

/// Represents a board that the cells inhabit.
pub struct Board {
    board: HashSet<GlobalPosition>,
}

/// The x & y positions of a [`Cell`] on the board.
#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct GlobalPosition {
    x: i32,
    y: i32,
}

impl GlobalPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

pub enum Cell {
    Alive,
    Dead,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            board: HashSet::new(),
        }
    }
}

impl Board {
    pub fn set_alive(&mut self, position: GlobalPosition) {
        self.board.insert(position);
    }

    pub fn set_dead(&mut self, position: GlobalPosition) {
        self.board.remove(&position);
    }

    pub fn is_alive(&self, position: GlobalPosition) -> bool {
        self.board.contains(&position)
    }

    pub fn is_dead(&self, position: GlobalPosition) -> bool {
        !self.board.contains(&position)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// A cell will be dead unless it has been set to alive.
    fn dead_by_default() {
        let position = GlobalPosition::new(1, 1);
        let mut board = Board::default();

        assert!(board.is_dead(position));
    }

    #[test]
    /// Sets a cell to be alive.
    fn set_cell_alive() {
        let position = GlobalPosition::new(1, 1);
        let mut board = Board::default();

        board.set_alive(position);
        assert!(board.is_alive(position));
    }
}
