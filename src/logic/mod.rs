mod simplistic;

pub trait Simulator {
    fn update(&mut self);

    fn batch_update(&mut self, amount: u64) {
        for _ in 0..amount {
            self.update();
        }
    }

    fn set(&mut self, position: GlobalPosition, cell: Cell);

    fn get(&self, position: GlobalPosition) -> Cell;

    fn export(&self, position_one: GlobalPosition, position_two: GlobalPosition) {
        todo!()
    }

    fn export_file(&self) {
        todo!()
    }
}

#[derive(PartialEq, Debug, Clone, Copy)]
pub enum Cell {
    Alive,
    Dead,
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
