use std::collections::HashMap;

pub struct Board {
    current_board: Box<HashMap<Position, Cell>>,
    buffer_board: Box<HashMap<Position, Cell>>,
}

pub struct Position {
    x: u16,
    y: u16,
}

pub enum Cell {
    Alive,
    Dead,
}
