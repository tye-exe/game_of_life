//! Contains the essential data types & all [`Simulator`] implementations.

pub use display::BoardDisplay;
use std::{
    num::NonZeroU32,
    sync::{mpsc, Arc, Mutex},
};
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
    fn new(display: SharedDisplay) -> Self;

    /// Advances the simulation by one tick.
    fn tick(&mut self);

    /// Updates the board being displayed by the ui.
    fn update_display(&mut self);

    /// Sets the display area sent to the ui to the given area.
    fn set_display_area(&mut self, new_area: Area);

    /// Sets the cell at the given position on the board.
    fn set(&mut self, position: GlobalPosition, cell: Cell);

    /// Gets the cell at the given position on the board.
    fn get(&self, position: GlobalPosition) -> Cell;

    /// Gets the current generation of simulation.
    fn get_generation(&self) -> u64;

    /// Outputs the entire board as a [`BoardStore`].
    fn save_board(&self) -> BoardStore;

    /// Attempts to load a new board.
    ///
    /// The result of this attempt will be returned as a [`LoadStatus`].
    fn load_board(&mut self, board: BoardStore) -> LoadStatus;

    /// Outputs an area of the board as a [`BoardStore`].
    fn save_blueprint(&self, area: Area) -> BoardStore;

    /// Attempts to load a blueprint at the given position. The given position will be the top left of the loaded blueprint.
    ///
    /// The result of this attempt will be returned as a [`LoadStatus`].
    fn load_blueprint(
        &mut self,
        load_position: GlobalPosition,
        blueprint: BoardStore,
    ) -> LoadStatus;
}

/// The data packets that the UI will send to the simulator.
#[cfg_attr(any(test, debug_assertions), derive(Debug))]
pub enum UiPacket {
    /// Requests for a new display area to be rendered.
    DisplayArea { new_area: Area },
    /// Sets a cell on the board.
    Set {
        /// The position of the cell to set.
        position: GlobalPosition,
        /// The state of the cell to set.
        cell_state: Cell,
    },

    /// Requests for the simulation to send a save of the boards current state to the ui for handling.
    SaveBoard,
    /// Sends a board to the simulation for it to simulate.
    LoadBoard {
        /// The board state to load.
        board: BoardStore,
    },

    /// Requests for the simulation to send a save of a portion of the current board to the ui for handling.
    SaveBlueprint {
        /// The area to save.
        area: Area,
    },
    /// Sends a blueprint for the simulation to load.
    LoadBlueprint {
        /// The position of to load the blueprint at.
        /// The blueprint will be loaded with this position as the top left.
        load_position: GlobalPosition,
        /// The blueprint to load.
        blueprint: BoardStore,
    },

    /// Starts the simulation.
    Start,
    /// Starts the simulation, with it automatically stopping at the given generation.
    StartUntil { generation: u64 },
    /// Stops the simulation.
    Stop,

    /// Sets the current speed of the simulation.
    SimulationSpeed { speed: SimulationSpeed },

    /// Terminates the simulator thread.
    /// This is unrecoverable without relaunching the application.
    Terminate,
}

/// The data packets that the simulator will send to the ui.
#[cfg_attr(any(test, debug_assertions), derive(Debug))]
pub enum SimulatorPacket {
    /// A save of the boards current state.
    BoardSave { board: BoardStore },
    /// The result of attempting to load a board.
    BoardLoadResult { status: LoadStatus },

    /// A save of a portion of the board.
    BlueprintSave { blueprint: BoardStore },
    /// The result of attempting to load a blueprint.
    BlueprintLoadResult { status: LoadStatus },
}

#[cfg_attr(any(test, debug_assertions), derive(Debug))]
pub struct BoardStore {}

#[cfg_attr(any(test, debug_assertions), derive(Debug))]
pub enum LoadStatus {
    Success,
    Fail,
}

#[cfg_attr(any(test, debug_assertions), derive(Debug))]
pub struct SimulationSpeed {
    ticks_per_second: Option<NonZeroU32>,
}
impl SimulationSpeed {
    pub const UNCAPPED: Self = {
        Self {
            ticks_per_second: None,
        }
    };

    pub fn new(ticks_per_second: u32) -> Self {
        Self {
            ticks_per_second: Some(
                NonZeroU32::new(ticks_per_second)
                    .unwrap_or(unsafe { NonZeroU32::new_unchecked(10) }),
            ),
        }
    }

    /// Gets the ticks per second the simulation will run at.
    /// If [`None`] is returned there is no cap for the simulation speed.
    pub fn get(&self) -> Option<NonZeroU32> {
        self.ticks_per_second
    }
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
    #[derive(Clone, Copy)]
    #[cfg_attr(any(test, debug_assertions), derive(Debug))]
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
