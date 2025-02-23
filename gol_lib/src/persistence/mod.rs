//! Contains the data structures used for handling blueprint & save data.
pub mod load;
pub mod preview;
pub mod save;

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::Duration,
};

pub(crate) use load::load;
pub use load::{load_board_data, ParseError};
pub use preview::load_preview;
pub use save::SaveBuilder;

use crate::{Area, GlobalPosition};
use bitvec::boxed::BitBox;

/// The latest supported save format version.
const CURRENT_SAVE_VERSION: u16 = 0;

/// The board data that a simulation consists of.
#[derive(serde::Deserialize, serde::Serialize)]
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Default))]
pub struct SimulationSave {
    pub(crate) generation: u64,
    pub(crate) board_area: Area,
    pub(crate) board_data: BitBox,
}

impl SimulationSave {
    pub fn new(generation: u64, board_area: Area, board_data: impl Into<BitBox>) -> Self {
        Self {
            generation,
            board_area,
            board_data: board_data.into(),
        }
    }
}

/// The board data that a blueprint consists of.
#[derive(serde::Deserialize)]
#[cfg_attr(any(test, debug_assertions), derive(Debug))]
pub struct SimulationBlueprint {
    pub(crate) x_size: i32,
    pub(crate) y_size: i32,
    pub(crate) blueprint_data: BitBox,
}

impl SimulationBlueprint {
    pub fn new(x_size: i32, y_size: i32, blueprint_data: impl Into<BitBox>) -> Self {
        Self {
            x_size,
            y_size,
            blueprint_data: blueprint_data.into(),
        }
    }
}

/// The data that a save of a simulation consists of.
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(any(test), derive(Debug, PartialEq))]
pub(crate) struct SaveData {
    version: u16,

    name: Box<str>,
    description: Box<str>,
    tags: Box<[Box<str>]>,

    time: Duration,
    view_position: Option<GlobalPosition>,

    #[serde(flatten)]
    simulation_save: SimulationSave,
}

/// Generates the filename (including extension) of the save file from the save file content.
fn generate_filename(
    board_area: &Area,
    save_name: &str,
    save_description: &str,
    save_tags: &[Box<str>],
    save_time: &Duration,
) -> String {
    let mut hasher = DefaultHasher::new();
    save_name.hash(&mut hasher);
    save_description.hash(&mut hasher);
    board_area.hash(&mut hasher);
    save_time.hash(&mut hasher);
    save_tags.hash(&mut hasher);

    let mut filename = hasher.finish().to_string();
    filename.push_str(".save");
    filename
}
