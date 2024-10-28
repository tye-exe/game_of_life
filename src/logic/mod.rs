use std::sync::{mpsc, Arc};

use display::BoardDisplay;

// mod simplistic;

pub trait Simulator {
    fn update(&mut self);

    fn batch_update(&mut self, amount: u64) {
        for _ in 0..amount {
            self.update();
        }
    }

    fn set(&mut self, position: GlobalPosition, cell: Cell);

    fn get(&self, position: GlobalPosition) -> Cell;

    fn export(&self, from: GlobalPosition, to: GlobalPosition) {
        todo!()
    }

    fn export_file(&self) {
        todo!()
    }

    // fn get_display_board(&self, from: GlobalPosition, to: GlobalPosition) -> BoardDisplay;
    // fn get_display_channel(&self)
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Cell {
    Alive,
    Dead,
}

/// The x & y positions of a [`Cell`] on the board.
#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub struct GlobalPosition {
    x: i32,
    y: i32,
}

impl GlobalPosition {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn get_x(&self) -> i32 {
        self.x
    }

    pub fn get_y(&self) -> i32 {
        self.y
    }
}

impl std::ops::Sub<(i32, i32)> for GlobalPosition {
    type Output = Self;

    fn sub(self, rhs: (i32, i32)) -> Self::Output {
        GlobalPosition::new(self.x - rhs.0, self.y - rhs.1)
    }
}

impl std::ops::Add<(i32, i32)> for GlobalPosition {
    type Output = Self;

    fn add(self, rhs: (i32, i32)) -> Self::Output {
        GlobalPosition::new(self.x + rhs.0, self.y + rhs.1)
    }
}

impl From<(i32, i32)> for GlobalPosition {
    fn from(value: (i32, i32)) -> Self {
        GlobalPosition {
            x: value.0,
            y: value.1,
        }
    }
}

pub mod display {
    use std::{
        cell::{Cell, RefCell},
        sync::Arc,
    };

    use super::GlobalPosition;

    #[derive(Clone, Copy)]
    enum BoardBuffer {
        BufferOne,
        BufferTwo,
    }

    pub type BoardDisplay = Arc<[Box<[super::Cell]>]>;
    // pub struct BoardDisplay

    /// A single wrapper struct around the two opposite corners of rectangle.
    pub struct Positions {
        pub to: GlobalPosition,
        pub from: GlobalPosition,
    }

    /// Allows for the bi-directional communication between the display & the [`Simulator`](super::Simulator).
    pub struct DisplayData {
        /// One corner of the displayed cells.
        to: Cell<GlobalPosition>,
        /// The opoosite corner of the displayed cells.
        from: Cell<GlobalPosition>,

        board_buffer_one: RefCell<BoardDisplay>,
        board_buffer_two: RefCell<BoardDisplay>,

        front_board: Cell<BoardBuffer>,
    }

    impl DisplayData {
        pub fn new(to: impl Into<GlobalPosition>, from: impl Into<GlobalPosition>) -> Self {
            let to = to.into();
            let from = from.into();

            let x = (to.get_x() - from.get_x()).unsigned_abs();
            let y = (to.get_y() - from.get_y()).unsigned_abs();

            // Generates an arary of arrays of dead cells.
            let board_buffer_one = std::iter::repeat_with(|| {
                // Generates an array of dead cells.
                std::iter::repeat(super::Cell::Dead)
                    .clone()
                    .take(y as usize)
                    .collect::<Box<[super::Cell]>>()
            })
            .take(x as usize)
            .collect::<BoardDisplay>();

            let board_buffer_two = board_buffer_one.clone();

            Self {
                to: Cell::new(to),
                from: Cell::new(from),
                board_buffer_one: RefCell::new(board_buffer_one),
                board_buffer_two: RefCell::new(board_buffer_two),
                front_board: Cell::new(BoardBuffer::BufferOne),
            }
        }

        pub fn set_coordinates(
            &self,
            to: impl Into<GlobalPosition>,
            from: impl Into<GlobalPosition>,
        ) {
            self.to.set(to.into());
            self.from.set(from.into());
        }

        pub fn get_coordinates(&self) -> Positions {
            Positions {
                to: self.to.get(),
                from: self.from.get(),
            }
        }

        pub fn get_board(&self) -> BoardDisplay {
            match self.front_board.get() {
                BoardBuffer::BufferOne => self.board_buffer_one.borrow().clone(),
                BoardBuffer::BufferTwo => self.board_buffer_two.borrow().clone(),
            }
        }

        pub fn set_board(&self, board: BoardDisplay) {
            match self.front_board.get() {
                BoardBuffer::BufferOne => {
                    // Try to update the board
                    match self.board_buffer_two.try_borrow_mut() {
                        Ok(mut val) => {
                            *val = board;

                            // Invert the front board
                            self.front_board.set(BoardBuffer::BufferTwo);
                        }
                        Err(err) => {
                            log::error!("Failed to set board: {err}");
                        }
                    }
                }
                BoardBuffer::BufferTwo => {
                    // Try to update the board
                    match self.board_buffer_one.try_borrow_mut() {
                        Ok(mut val) => {
                            *val = board;

                            // Invert the front board
                            self.front_board.set(BoardBuffer::BufferOne);
                        }
                        Err(err) => {
                            log::error!("Failed to set board: {err}");
                        }
                    }
                }
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        // Ensures that the logger has started before any tests are run.
        #[cfg(test)]
        #[ctor::ctor]
        fn init() {
            colog::init();
        }

        #[test]
        fn construction_test() {
            DisplayData::new((4, 4), (0, 0));
        }

        #[test]
        fn get_coords() {
            let display_data = DisplayData::new((4, 4), (0, 0));

            let positions = display_data.get_coordinates();
            assert_eq!(positions.to, (4, 4).into());
            assert_eq!(positions.from, (0, 0).into());
        }

        #[test]
        fn set_coords() {
            let display_data = DisplayData::new((4, 4), (0, 0));

            display_data.set_coordinates((5, 5), (1, 1));
            let positions = display_data.get_coordinates();
            assert_eq!(positions.to, (5, 5).into());
            assert_eq!(positions.from, (1, 1).into());
        }

        #[test]
        fn get_board() {
            let display_data = DisplayData::new((4, 4), (0, 0));

            // The entire board is dead
            let board = display_data.get_board();
            for array in board.iter() {
                for cell in array.iter() {
                    assert_eq!(*cell, crate::logic::Cell::Dead);
                }
            }
        }

        /// Generates DisplayData where one board is all dead & the other is all alive.
        fn display_different_board() -> DisplayData {
            use crate::logic::Cell;
            let display_data = DisplayData::new((4, 4), (0, 0));

            // Generates an arary of arrays of dead cells.
            let board = std::iter::repeat_with(|| {
                // Generates an array of dead cells.
                std::iter::repeat(Cell::Alive)
                    .clone()
                    .take(4usize)
                    .collect::<Box<[Cell]>>()
            })
            .take(4usize)
            .collect::<BoardDisplay>();

            display_data.set_board(board);
            display_data
        }

        #[test]
        fn set_board() {
            use crate::logic::Cell;
            let display_data = display_different_board();

            // The current board will be alive as it has been set so
            let set_board = display_data.get_board();
            for array in set_board.iter() {
                for cell in array.iter() {
                    assert_eq!(*cell, Cell::Alive);
                }
            }
        }

        #[test]
        fn boards_are_different() {
            use crate::logic::Cell;
            let display_data = display_different_board();

            // Buf one (The board available to edit) will still be dead
            let buf_one = display_data.board_buffer_one.borrow().clone();
            for array in buf_one.iter() {
                for cell in array.iter() {
                    assert_eq!(*cell, Cell::Dead);
                }
            }

            // Buf two (The board that has been edited) will be alive
            let buf_two = display_data.board_buffer_two.borrow().clone();
            for array in buf_two.iter() {
                for cell in array.iter() {
                    assert_eq!(*cell, Cell::Alive);
                }
            }
        }
    }
}
