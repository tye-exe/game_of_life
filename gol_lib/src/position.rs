/// The x & y positions of a [`Cell`] on the Conways game of life board.
///
/// To move "right" on the board, the x must be increased.
/// To move "down" on the board, the y must be increased.
/// The opposites also apply.
#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub struct GlobalPosition {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl GlobalPosition {
    /// Creates a new [`GlobalPosition`] at the given x & y coordinates.
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    /// Gets the represented x position.
    pub fn get_x(&self) -> i32 {
        self.x
    }

    /// Gets the represented y position.
    pub fn get_y(&self) -> i32 {
        self.y
    }
}

impl<T: Into<GlobalPosition>> std::ops::Sub<T> for GlobalPosition {
    type Output = Self;

    fn sub(self, other_position: T) -> Self::Output {
        let other_position: GlobalPosition = other_position.into();
        GlobalPosition::new(self.x - other_position.x, self.y - other_position.y)
    }
}

impl<T: Into<GlobalPosition>> std::ops::Add<T> for GlobalPosition {
    type Output = Self;

    fn add(self, other_position: T) -> Self::Output {
        let other_position: GlobalPosition = other_position.into();
        GlobalPosition::new(self.x + other_position.x, self.y + other_position.y)
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
