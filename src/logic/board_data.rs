//! Contains the data structures used for handling blueprint & save data.

use super::{Area, GlobalPosition};
use bitvec::boxed::BitBox;
use std::path::Path;

/// The board data that a simulation consists of.
#[derive(serde::Deserialize)]
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq))]
pub(crate) struct SimulationSave {
    pub(crate) generation: u64,
    pub(crate) board_area: Area,
    pub(crate) board_data: BitBox,
}

impl SimulationSave {
    pub(crate) fn new(generation: u64, board_area: Area, board_data: impl Into<BitBox>) -> Self {
        Self {
            generation,
            board_area,
            board_data: board_data.into(),
        }
    }
}

/// The data that a save of a simulation consists of.
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq))]
pub(crate) struct BoardSave {
    version: u32,

    save_name: Option<Box<str>>,
    save_description: Option<Box<str>>,
    view_position: Option<GlobalPosition>,

    generation: u64,
    board_area: Area,
    board_data: BitBox,
}

impl BoardSave {
    pub(crate) fn new(
        save_name: Option<Box<str>>,
        save_description: Option<Box<str>>,
        view_position: Option<GlobalPosition>,
        simulation_save: SimulationSave,
    ) -> Self {
        Self {
            version: 0,
            save_name,
            save_description,
            view_position,
            generation: simulation_save.generation,
            board_area: simulation_save.board_area,
            board_data: simulation_save.board_data,
        }
    }

    pub fn save(self) {
        todo!()
    }

    pub fn load(path: &Path) -> Result<Self, ()> {
        todo!()
    }
}

/// The board data that a blueprint consists of.
#[derive(serde::Deserialize)]
#[cfg_attr(any(test, debug_assertions), derive(Debug))]
pub(crate) struct SimulationBlueprint {
    pub(crate) x_size: i32,
    pub(crate) y_size: i32,
    pub(crate) blueprint_data: BitBox,
}

impl SimulationBlueprint {
    pub(crate) fn new(x_size: i32, y_size: i32, blueprint_data: impl Into<BitBox>) -> Self {
        Self {
            x_size,
            y_size,
            blueprint_data: blueprint_data.into(),
        }
    }
}
