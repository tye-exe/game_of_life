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

    display: SharedDisplay,

    sender: mpsc::Sender<SimulatorPacket>,
    ui_receiver: UiReceiver,

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

    fn new(ui_receiver: UiReceiver, display: SharedDisplay) -> (SimulatorReceiver, Self) {
        let (sender, receiver) = mpsc::channel();

        (
            receiver,
            Self {
                board: Default::default(),
                display,
                sender,
                ui_receiver,
                display_size_buf: Default::default(),
            },
        )
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
        for x in self.display_size_buf.get_to().get_x()..self.display_size_buf.get_from().get_x() {
            let mut y_builder = Vec::new();
            for y in self.display_size_buf.get_to().get_y()..self.display_size_buf.get_to().get_y()
            {
                y_builder.push(self.get((x, y).into()));
            }
            // Convert the vec into the correct type
            let array: Box<[Cell]> = y_builder.into();
            board_build.push(array);
        }

        // Updates the board to display.
        *display = Some(board_build.into());
    }

    fn ui_communication(&mut self) {
        loop {
            // Process all sent packets.
            use std::sync::mpsc::TryRecvError;
            let ui_packet = match self.ui_receiver.try_recv() {
                Ok(ui_packet) => ui_packet,
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    std::panic!("UI closed communication!");
                }
            };

            match ui_packet {
                UiPacket::DisplayArea { new_area } => self.display_size_buf = new_area,
            }
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
        // let board = Board::default();

        // assert_eq!(board.get(position), Cell::Dead);
    }

    #[test]
    /// Sets a cell to be alive.
    fn set_cell_alive() {
        let position = GlobalPosition::new(1, 1);
        // let mut board = Board::default();

        // board.set(position, Cell::Alive);
        // assert_eq!(board.get(position), Cell::Alive);
    }

    // #[test]
    // fn simulates_correctly() {

    // }
}
