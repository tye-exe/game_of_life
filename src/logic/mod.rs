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

/// A cell on the board.
pub enum Cell {
    Alive,
    Dead,
}
