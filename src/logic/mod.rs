//! Contains the essential data types & all [`Simulator`] implementations.

pub use display::BoardDisplay;
use std::sync::{mpsc, Arc, Mutex};
pub use types::{Area, Cell, GlobalPosition};

mod display;

/// A simplistic implementation of [`Simulator`].
/// There is no consideration to performance; Only a Minimum Viable Product.
pub mod simplistic;

/// A pointer to the [`Mutex`] used to share the display board.
/// The time either the ui or the [`Simulator`] will hold a lock on the [`Mutex`] is not guaranteed.
pub type SharedDisplay = Arc<Mutex<Option<BoardDisplay>>>;

/// The [`Receiver`] for [`UiPacket`]s from the ui.
///
/// [`Receiver`]: std::sync::mpsc::Receiver
pub type UiReceiver = mpsc::Receiver<UiPacket>;
/// The [`Sender`] for [`UiPacket`]s being sent from the ui.
/// Only the ui should ever have this [`Sender`].
///
/// [`Sender`]: std::sync::mpsc::Sender
pub type UiSender = mpsc::Sender<UiPacket>;
/// The [`Receiver`] for [`SimulatorPacket`]s from the [`Simulator`].
///
/// [`Receiver`]: std::sync::mpsc::Receiver
pub type SimulatorReceiver = mpsc::Receiver<SimulatorPacket>;
/// The [`Sender`] for [`SimulatorPacket`]s being sent from the [`Simulator`].
/// Only the [`Simulator`] should ever have this [`Sender`].
///
/// [`Sender`]: std::sync::mpsc::Sender
pub type SimulatorSender = mpsc::Sender<SimulatorPacket>;

/// Creates the channels for communication between the [`Simulator`] & the UI.
pub fn create_channels() -> ((UiSender, UiReceiver), (SimulatorSender, SimulatorReceiver)) {
    (mpsc::channel(), mpsc::channel())
}

/// An implementation of [`Simulator`] can simulate Conways game of life.
///
/// Each implementation is guaranteed to correctly simulate Conways game of life, however the performance of any
/// impulmentation is not guaranteed
pub trait Simulator {
    /// Creates a new simulator.
    fn new(
        display: SharedDisplay,
        ui_receiver: UiReceiver,
        simulator_sender: SimulatorSender,
    ) -> Self;

    /// Advances the simulation by one tick.
    fn tick(&mut self);

    /// Updates the board being displayed by the ui.
    fn update_display(&mut self);

    /// Handles communication between the ui & the simulation.
    fn ui_communication(&mut self);

    /// Sets the cell at the given position on the board.
    fn set(&mut self, position: GlobalPosition, cell: Cell);

    /// Gets the cell at the given position on the board.
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
    /// Represents the state of a cell within the Conways game of life simulation.
    #[derive(PartialEq, Debug, Clone, Copy)]
    pub enum Cell {
        Alive,
        Dead,
    }

    impl Default for Cell {
        fn default() -> Self {
            Cell::Dead
        }
    }

    /// The x & y positions of a [`Cell`] on the Conways game of life board.
    #[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
    pub struct GlobalPosition {
        x: i32,
        y: i32,
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
        /// The small x & the small y position.
        from: GlobalPosition,
        /// The big x & the big y position.
        to: GlobalPosition,
    }

    impl Default for Area {
        /// Constructs a new [`Area`], with zero size.
        fn default() -> Self {
            Self::new((0, 0), (0, 0))
        }
    }

    impl Area {
        /// Constructs a new [`Area`] covering from the small x & y to the large x & y.
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
        /// Tests that the fields within the area struct are correctly sorted into the smallest x & y and into the
        /// largest x & y respectively.
        fn from_lower_to_higher() {
            let area = Area::new((10, 5), (5, 10));

            assert_eq!(area.get_from(), (5, 5).into());
            assert_eq!(area.get_to(), (10, 10).into());
        }
    }
}
