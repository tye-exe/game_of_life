//! Contains the data structures used for handling blueprint & save data.
pub mod load;
pub mod preview;
pub mod save;

use std::{
    hash::{DefaultHasher, Hash, Hasher},
    time::Duration,
};

pub use load::{
    ParseError, load_blueprint, load_blueprint_preview, load_board_data, load_preview,
    load_save_preview,
};
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
#[derive(serde::Deserialize, serde::Serialize)]
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Default))]
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

/// The data that the save format consists of.
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(any(test), derive(Debug, PartialEq))]
pub struct SaveFormat<Data> {
    /// The save format version.
    version: u16,

    /// The name for the save data (not the filename).
    name: Box<str>,
    /// A description of the save data.
    description: Box<str>,
    /// The tags for the save data.
    tags: Box<[Box<str>]>,
    /// Time time this save data was made.
    time: Duration,

    /// Generic save data.
    #[serde(flatten)]
    data: Data,
}

/// The data structure for a save of the board.
/// Used in [`SaveFormat`].
#[derive(serde::Deserialize, serde::Serialize)]
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Default))]
pub struct Save {
    /// The position that the user is viewing the save at.
    view_position: Option<GlobalPosition>,
    #[serde(flatten)]
    simulation_save: SimulationSave,
}

/// The data structure for a save of a blueprint.
/// Used in [`SaveFormat`].
#[derive(serde::Deserialize, serde::Serialize)]
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq, Default))]
pub struct Blueprint {
    #[serde(flatten)]
    blueprint: SimulationBlueprint,
}

/// For generating the filenames using the generic save data for [`SaveFormat`] from [`SaveBuilder`].
pub trait GenerateName {
    /// Generates the filename (including extension) of the file from the data stored in the file.
    fn filename(
        &self,
        save_name: &str,
        save_description: &str,
        save_tags: &[Box<str>],
        save_time: &Duration,
    ) -> String;
}

impl GenerateName for Save {
    fn filename(
        &self,
        save_name: &str,
        save_description: &str,
        save_tags: &[Box<str>],
        save_time: &Duration,
    ) -> String {
        let mut hasher = DefaultHasher::new();

        save_name.hash(&mut hasher);
        save_description.hash(&mut hasher);
        self.simulation_save.board_area.hash(&mut hasher);
        save_time.hash(&mut hasher);
        save_tags.hash(&mut hasher);

        let mut filename = hasher.finish().to_string();
        filename.push_str(".save");
        filename
    }
}

impl GenerateName for Blueprint {
    fn filename(
        &self,
        save_name: &str,
        save_description: &str,
        save_tags: &[Box<str>],
        save_time: &Duration,
    ) -> String {
        let mut hasher = DefaultHasher::new();

        save_name.hash(&mut hasher);
        save_description.hash(&mut hasher);
        self.blueprint.x_size.hash(&mut hasher);
        self.blueprint.y_size.hash(&mut hasher);
        save_time.hash(&mut hasher);
        save_tags.hash(&mut hasher);

        let mut filename = hasher.finish().to_string();
        filename.push_str(".save");
        filename
    }
}

impl Save {
    /// Generates the filename from its component parts.
    pub fn generate_filename(
        board_area: Area,
        save_name: &str,
        save_description: &str,
        save_tags: &[Box<str>],
        save_time: &Duration,
    ) -> String {
        Self {
            view_position: None,
            simulation_save: SimulationSave {
                board_area,
                ..Default::default()
            },
        }
        .filename(save_name, save_description, save_tags, save_time)
    }
}

impl Blueprint {
    /// Generates the filename from its component parts.
    pub fn generate_filename(
        x_size: i32,
        y_size: i32,
        blueprint_name: &str,
        blueprint_description: &str,
        blueprint_tags: &[Box<str>],
        blueprint_time: &Duration,
    ) -> String {
        Self {
            blueprint: SimulationBlueprint {
                x_size,
                y_size,
                blueprint_data: Default::default(),
            },
        }
        .filename(
            blueprint_name,
            blueprint_description,
            blueprint_tags,
            blueprint_time,
        )
    }
}
