use std::sync::{mpsc, Arc, Mutex};
use types::{Area, Cell, GlobalPosition};

pub mod simplistic;

pub type BoardDisplay = Arc<[Box<[Cell]>]>;

pub type UiReceiver = mpsc::Receiver<UiPacket>;
pub type SharedDisplay = Arc<Mutex<Option<BoardDisplay>>>;
pub type SimulatorReceiver = mpsc::Receiver<SimulatorPacket>;

pub trait Simulator {
    fn new(ui_receiver: UiReceiver, display: SharedDisplay) -> (SimulatorReceiver, Self);

    fn update(&mut self);

    fn set(&mut self, position: GlobalPosition, cell: Cell);

    fn get(&self, position: GlobalPosition) -> Cell;

    fn export(&self, area: Area) {
        todo!()
    }

    fn export_file(&self) {
        todo!()
    }

    // fn get_display_board(&self, from: GlobalPosition, to: GlobalPosition) -> BoardDisplay;
    // fn get_display_channel(&self)
}

/// The data packets that the UI will send to the simulator.
pub enum UiPacket {
    /// Requests for a new display area to be rendered.
    DisplayArea { new_area: Area },
}

/// The data packets that the simulator will send to the ui.
pub enum SimulatorPacket {
    ToDo,
}

/// A module containing shared data types, the data types are in a separate module to force sub-modules
/// to use the public interface provided by the data types.
mod types {
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
    pub struct Area {
        from: GlobalPosition,
        to: GlobalPosition,
    }

    impl Default for Area {
        /// Constructs a new [`Area`], with zero size.
        fn default() -> Self {
            Self::new((0, 0), (0, 0))
        }
    }

    impl Area {
        /// Constructs a new [`Area`].
        pub fn new(pos1: impl Into<GlobalPosition>, pos2: impl Into<GlobalPosition>) -> Self {
            let pos1 = pos1.into();
            let pos2 = pos2.into();

            // Construct from with the smallest x & y
            let from = GlobalPosition {
                x: pos1.get_x().min(pos2.get_x()),
                y: pos1.get_y().min(pos2.get_y()),
            };
            // Construct to with the biggest x & y
            let to = GlobalPosition {
                x: pos1.get_x().max(pos2.get_x()),
                y: pos1.get_y().max(pos2.get_y()),
            };

            Self { from, to }
        }

        /// Gets the smallest x & smallest y of the area.
        pub fn get_from(&self) -> GlobalPosition {
            self.from
        }

        /// Gets the biggest x & biggest y of the area.
        pub fn get_to(&self) -> GlobalPosition {
            self.to
        }
    }

    #[cfg(test)]
    mod area_tests {
        use super::*;

        #[test]
        fn from_lower_to_higher() {
            let area = Area::new((10, 5), (5, 10));

            assert_eq!(area.get_from(), (5, 5).into());
            assert_eq!(area.get_to(), (10, 10).into());
        }
    }
}
