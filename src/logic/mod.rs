use std::collections::HashMap;

/// The x & y dimensions of the [`CellChunk`].
pub const CHUNK_SIZE: u16 = 16;

/// The internal array size for [`CellChunk`].
/// It is one less than [`CHUNK_SIZE`] due to arrays starting at index 0.
const CHUNK_ARRAY_SIZE: usize = (CHUNK_SIZE - 1) as usize;

/// Represents a board that the cells inhabit.
pub struct Board {
    board: HashMap<ChunkPosition, CellChunk>,
}

/// The x & y positions of a [`CellChunk`] on the board.
#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct ChunkPosition {
    x: u16,
    y: u16,
}

impl ChunkPosition {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

/// The x & y positions of a [`Cell`] on the board.
#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct GlobalPosition {
    x: u16,
    y: u16,
}

impl GlobalPosition {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
}

/// The x & y positions of a [`Cell`] within a [`CellChunk`].
#[derive(Eq, Hash, PartialEq, Clone, Copy)]
pub struct LocalPosition {
    x: usize,
    y: usize,
}

impl LocalPosition {
    pub fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
}

impl From<GlobalPosition> for ChunkPosition {
    fn from(value: GlobalPosition) -> Self {
        ChunkPosition {
            x: value.x / CHUNK_SIZE,
            y: value.y / CHUNK_SIZE,
        }
    }
}

impl From<GlobalPosition> for LocalPosition {
    fn from(value: GlobalPosition) -> Self {
        LocalPosition {
            x: (value.x % CHUNK_SIZE) as usize,
            y: (value.y % CHUNK_SIZE) as usize,
        }
    }
}

/// Contains a [`CHUNK_SIZE`] by [`CHUNK_SIZE`] grid of cells that comprise a sub-section of a
/// [`Board`].
struct CellChunk {
    position: ChunkPosition,
    chunk_data: [[Cell; CHUNK_ARRAY_SIZE]; CHUNK_ARRAY_SIZE],
}

impl CellChunk {
    /// Create a new [`CellChunk`] at the given position, where all the cells are dead.
    pub fn new_empty(position: ChunkPosition) -> Self {
        Self {
            position,
            chunk_data: Default::default(),
        }
    }

    pub fn set(&mut self, position: LocalPosition, cell: Cell) {
        self.chunk_data[position.x][position.y] = cell;
    }

    pub fn get(&self, position: LocalPosition) -> Cell {
        self.chunk_data[position.x][position.y]
    }
}

/// A cell on the board.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Cell {
    Alive,
    Dead,
}

impl Default for Cell {
    fn default() -> Self {
        Cell::Dead
    }
}

impl Default for Board {
    fn default() -> Self {
        todo!()
    }
}

impl Board {
    pub fn set(&mut self, position: GlobalPosition, cell: Cell) {
        self.board
            .entry(position.into())
            .or_insert(CellChunk::new_empty(position.into()))
            .set(position.into(), cell);
    }

    pub fn get(&self, position: GlobalPosition) -> Cell {
        match self.board.get(&position.into()) {
            Some(chunk) => chunk.get(position.into()),
            None => Cell::Dead,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Sets a cell to be alive
    fn set_cell_alive() {
        let mut board = Board::default();
        board.set(ChunkPosition::new(0, 0), Cell::Alive);
        let cell = board.get(ChunkPosition::new(0, 0));
        assert_eq!(cell, Cell::Alive);
    }
}
