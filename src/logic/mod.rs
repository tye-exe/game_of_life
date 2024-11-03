use std::sync::{mpsc, Arc};

pub type BoardDisplay = Arc<[Box<[Cell]>]>;
pub mod simplistic;

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

/// A single wrapper struct around the two opposite corners of rectangle.
pub struct Positions {
    pub to: GlobalPosition,
    pub from: GlobalPosition,
}
