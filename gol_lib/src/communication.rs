use std::num::NonZeroU32;

use crate::{
    board_data::{SimulationBlueprint, SimulationSave},
    Area, Cell, GlobalPosition,
};

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
        board: SimulationSave,
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
        blueprint: SimulationBlueprint,
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
    BoardSave { board: SimulationSave },

    /// A save of a portion of the board.
    BlueprintSave { blueprint: SimulationBlueprint },
}

#[cfg_attr(any(test, debug_assertions), derive(Debug))]
pub struct SimulationSpeed {
    pub(crate) ticks_per_second: Option<NonZeroU32>,
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
