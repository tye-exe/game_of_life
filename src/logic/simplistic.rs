use std::{
    collections::{HashMap, HashSet},
    ops::AddAssign,
};

use super::{Cell, GlobalPosition, Simulator};

/// Represents a board that the cells inhabit.
pub struct Board {
    board: HashSet<GlobalPosition>,
}

impl Default for Board {
    fn default() -> Self {
        Board {
            board: HashSet::new(),
            // working_board: HashSet::new(),
        }
    }
}

impl Simulator for Board {
    fn update(&mut self) {
        let mut seen = HashMap::new();
        let mut die = HashSet::new();

        self.board.iter().for_each(|position| {
            let position = *position;
            let mut surrounding = 0u8;
            surrounding += self.board.contains(&(position + (1, 1))) as u8;
            surrounding += self.board.contains(&(position + (1, 0))) as u8;
            surrounding += self.board.contains(&(position + (1, -1))) as u8;

            surrounding += self.board.contains(&(position + (0, 1))) as u8;
            surrounding += self.board.contains(&(position + (0, -1))) as u8;

            surrounding += self.board.contains(&(position + (-1, 1))) as u8;
            surrounding += self.board.contains(&(position + (-1, 0))) as u8;
            surrounding += self.board.contains(&(position + (-1, -1))) as u8;

            match surrounding {
                // Under population
                0 | 1 => {
                    die.insert(position);
                }
                2 | 3 => {}
                // Over population
                _ => {
                    die.insert(position);
                }
            }

            seen.entry(position + (1, 1)).or_insert(0u8).add_assign(1);
            seen.entry(position + (1, 0)).or_insert(0u8).add_assign(1);
            seen.entry(position + (1, -1)).or_insert(0u8).add_assign(1);

            seen.entry(position + (0, 1)).or_insert(0u8).add_assign(1);
            seen.entry(position + (0, -1)).or_insert(0u8).add_assign(1);

            seen.entry(position + (-1, 1)).or_insert(0u8).add_assign(1);
            seen.entry(position + (-1, 0)).or_insert(0u8).add_assign(1);
            seen.entry(position + (-1, -1)).or_insert(0u8).add_assign(1);
        });

        for position in die {
            self.board.remove(&position);
        }

        for (position, alive_neighbours) in seen {
            match alive_neighbours {
                // Under population
                0 | 1 => {
                    self.board.remove(&position);
                }
                2 | 3 => {}
                // Over population
                _ => {
                    self.board.remove(&position);
                }
            }
        }
    }

    fn set(&mut self, position: GlobalPosition, cell: Cell) {
        match cell {
            Cell::Alive => {
                self.board.insert(position);
            }
            Cell::Dead => {
                self.board.remove(&position);
            }
        };
    }

    fn get(&self, position: GlobalPosition) -> Cell {
        match self.board.contains(&position) {
            true => Cell::Alive,
            false => Cell::Dead,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// A cell will be dead unless it has been set to alive.
    fn dead_by_default() {
        let position = GlobalPosition::new(1, 1);
        let board = Board::default();

        assert_eq!(board.get(position), Cell::Dead);
    }

    #[test]
    /// Sets a cell to be alive.
    fn set_cell_alive() {
        let position = GlobalPosition::new(1, 1);
        let mut board = Board::default();

        board.set(position, Cell::Alive);
        assert_eq!(board.get(position), Cell::Alive);
    }

    #[test]
    fn simulates_correctly() {}
}
