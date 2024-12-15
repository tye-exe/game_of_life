use std::{
    collections::{HashMap, HashSet},
    ops::AddAssign,
    sync::{mpsc, Arc, Mutex},
};

use super::{
    Area, BoardDisplay, Cell, GlobalPosition, SharedDisplay, Simulator, SimulatorPacket,
    SimulatorReceiver, UiPacket, UiReceiver,
};

/// Represents a board that the cells inhabit.
pub struct Board {
    board: HashSet<GlobalPosition>,
    generation: u64,

    display: SharedDisplay,
    display_size_buf: Area,
}

impl Simulator for Board {
    fn tick(&mut self) {
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

        self.generation += 1;
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

    fn update_display(&mut self) {
        // Attempts to acquire the lock on the display.
        // If a lock could not be acquired the method returns early.
        use std::sync::TryLockError;
        let mut display = match self.display.try_lock() {
            Ok(display) => display,
            Err(TryLockError::WouldBlock) => {
                return;
            }
            Err(TryLockError::Poisoned(_)) => {
                core::panic!("Ui panicked!");
            }
        };

        // If the ui has not taken the display return early.
        if display.is_some() {
            return;
        }

        // Get the state of the board within the specified size
        let mut board_build = Vec::new();

        let from = &self.display_size_buf.get_from();
        let to = &self.display_size_buf.get_to();
        for x in from.get_x()..to.get_x() {
            let mut y_builder = Vec::new();
            for y in from.get_y()..to.get_y() {
                y_builder.push(self.get((x, y).into()));
            }
            // Convert the vec into the correct type
            let array: Box<[Cell]> = y_builder.into();
            board_build.push(array);
        }

        // Updates the board to display.
        *display = Some(BoardDisplay::new(self.generation, board_build));
    }

    fn new(display: SharedDisplay) -> Self {
        Self {
            board: Default::default(),
            display,
            display_size_buf: Default::default(),
            generation: 0,
        }
    }

    fn set_display_area(&mut self, new_area: Area) {
        self.display_size_buf = new_area;
    }

    fn get_generation(&self) -> u64 {
        self.generation
    }

    fn save_board(&self) -> super::BoardStore {
        todo!()
    }

    fn load_board(&mut self, board: super::BoardStore) -> super::LoadStatus {
        todo!()
    }

    fn save_blueprint(&self, area: Area) -> super::BoardStore {
        todo!()
    }

    fn load_blueprint(
        &mut self,
        load_position: GlobalPosition,
        blueprint: super::BoardStore,
    ) -> super::LoadStatus {
        todo!()
    }

    fn clear(&mut self) {
        self.board = HashSet::new();
        self.generation = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// A cell will be dead unless it has been set to alive.
    fn dead_by_default() {
        let board = Board::new(Default::default());
        for x in -10..=10 {
            for y in -10..=10 {
                assert_eq!(board.get((x, y).into()), Cell::Dead)
            }
        }
    }

    #[test]
    /// Sets a cell to be alive.
    fn set_cell_alive() {
        let position = GlobalPosition::new(1, 1);
        let mut board = Board::new(Default::default());

        board.set(position, Cell::Alive);
        assert_eq!(board.get(position), Cell::Alive);
    }

    /// Returns an iterator over [`Cell`], which gives "[`Cell::Alive`], [`Cell::Dead`]" in that order, forever.
    fn generate_cell_iterator() -> std::iter::FromFn<impl FnMut() -> Option<Cell>> {
        let mut generated_cell = Cell::Dead;
        let cell_iter = std::iter::from_fn(move || {
            generated_cell = match generated_cell {
                Cell::Alive => Cell::Dead,
                Cell::Dead => Cell::Alive,
            };
            Some(generated_cell)
        });
        cell_iter
    }

    #[test]
    /// Sets a pattern of alive & dead cells.
    fn set_cell_pattern() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        // Populate board
        let mut cell_iter = generate_cell_iterator();
        for x in -10..=10 {
            for y in -10..=10 {
                let cell = cell_iter.next().unwrap();
                board.set((x, y).into(), cell)
            }
        }

        // Read board
        let mut cell_iter = generate_cell_iterator();
        for x in -10..=10 {
            for y in -10..=10 {
                let cell = cell_iter.next().unwrap();
                let get = board.get((x, y).into());
                assert_eq!(cell, get);
            }
        }
    }

    #[test]
    /// The derived display will correctly represent the board.
    fn generates_correct_display() {
        let display: SharedDisplay = Default::default();
        let mut board = Board::new(display.clone());

        // Populate board
        let mut cell_iter = generate_cell_iterator();
        for y in -10..=10 {
            for x in -10..=10 {
                board.set((x, y).into(), cell_iter.next().unwrap());
            }
        }

        // Display init
        board.set_display_area(Area::new((-10, -10), (10, 10)));
        board.update_display();

        let mut mutex_guard = display.lock().unwrap();
        let take = mutex_guard.take();
        assert!(take.is_some());

        // Generate expected result
        let var_name = {
            use Cell::{Alive, Dead};
            let mut vec = Vec::new();

            for _ in 0..10 {
                let a: Box<[Cell]> = Box::new([
                    Alive, Dead, Alive, Dead, Alive, Dead, Alive, Dead, Alive, Dead, Alive, Dead,
                    Alive, Dead, Alive, Dead, Alive, Dead, Alive, Dead,
                ]);
                let b = Box::new([
                    Dead, Alive, Dead, Alive, Dead, Alive, Dead, Alive, Dead, Alive, Dead, Alive,
                    Dead, Alive, Dead, Alive, Dead, Alive, Dead, Alive,
                ]);
                vec.push(a);
                vec.push(b);
            }

            vec
        };

        let board_display = BoardDisplay::new(0, var_name);
        assert_eq!(board_display, take.unwrap())
    }
}
