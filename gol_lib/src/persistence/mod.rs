//! Contains the data structures used for handling blueprint & save data.
pub mod board_load;
pub mod board_save;
pub mod preview;

use std::{
    fs::File,
    path::Path,
    time::{Duration, SystemTime},
};

pub use board_load::load_save;
pub use board_save::SaveBuilder;
pub use preview::load_preview;
use serde::de::DeserializeOwned;

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

/// The errors that can occur when attempting to parse data from a file.
#[derive(thiserror::Error, Debug)]
#[cfg_attr(test, derive(kinded::Kinded))]
pub enum ParseError {
    /// Unable to read file.
    #[error("Unable to read save file: {0}")]
    FileParse(#[from] std::io::Error),
    /// The file contains invalid data.
    #[error("File '{path:?}' is not a valid save file: {serde_error}")]
    InvalidData {
        serde_error: serde_json::Error,
        path: Box<Path>,
    },
}

impl ParseError {
    /// The path to the file that caused the error, if it is available.
    pub fn file_path(&self) -> Option<&Path> {
        match self {
            ParseError::FileParse(..) => None,
            ParseError::InvalidData { path, .. } => Some(&**path),
        }
    }
}

/// Finds and parses `Data` instances from the given directory.
///
/// Returns `Err` if the given directory cannot be read from.
/// Otherwise an array of parsed data/errors will be returned.
fn load<'a, Data: DeserializeOwned>(
    save_location: impl Into<&'a Path>,
) -> Result<Box<[Result<Data, ParseError>]>, std::io::Error> {
    let parsed_data = std::fs::read_dir(save_location.into())?
        .into_iter()
        // Try to read files
        .filter_map(|dir_content| {
            // Only try to parse files
            match dir_content {
                Ok(content) => match content.file_type() {
                    Ok(file_type) => {
                        if file_type.is_file() {
                            Some(Ok(content))
                        } else {
                            None
                        }
                    }
                    // Cannot read file type
                    Err(err) => Some(Err(ParseError::FileParse(err))),
                },
                // Cannot read file
                Err(err) => Some(Err(ParseError::FileParse(err))),
            }
        })
        // Parse file content
        .map(|file| {
            let file = file?;
            let open = File::open(file.path())?;

            let content: Data =
                serde_json::from_reader(open).map_err(|err| ParseError::InvalidData {
                    serde_error: err,
                    path: file.path().into_boxed_path(),
                })?;

            Ok(content)
        })
        .collect();

    Ok(parsed_data)
}

/// The data that a save of a simulation consists of.
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(any(test), derive(Debug, PartialEq))]
pub struct SaveData {
    version: u16,

    name: Box<str>,
    description: Box<str>,
    tags: Box<[Box<str>]>,

    time: Duration,
    view_position: Option<GlobalPosition>,

    #[serde(flatten)]
    simulation_save: SimulationSave,
}

impl SaveData {
    pub fn version(&self) -> u16 {
        todo!()
    }

    pub fn name(&self) -> &str {
        todo!()
    }
    pub fn description(&self) -> &str {
        todo!()
    }
    pub fn time(&self) -> Duration {
        todo!()
    }
    pub fn view_position(&self) -> GlobalPosition {
        todo!()
    }
    pub fn simulation_save(&self) -> SimulationSave {
        todo!()
    }
    pub fn tags(&self) -> Box<[Box<str>]> {
        todo!()
    }
}

// #[derive(thiserror::Error, Debug)]
// pub enum LoadError {
//     #[error("Failed to find possible save files: {0}")]
//     FileSearch(#[from] walkdir::Error),
//     #[error("Failed to read save file: {0}")]
//     FileRead(#[from] std::io::Error),
// }

// /// Parses data from all files recursively in the given location.
// ///
// /// Any invalid files that cannot be parsed as [`Data`] will be ignored.
// pub fn load_files<Data: DeserializeOwned>(
//     save_location: impl Into<Box<Path>>,
// ) -> Result<Box<[Data]>, LoadError> {
//     let mut saves = Vec::new();

//     for file in WalkDir::new(save_location.into()) {
//         let file = file?;
//         if !file.file_type().is_file() {
//             continue;
//         }

//         let file_data = std::fs::read_to_string(file.into_path())?;
//         if let Ok(data) = serde_json::from_str(&file_data) {
//             saves.push(data);
//         }
//     }

//     Ok(saves.into_boxed_slice())
// }

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn load_a_save() {
//         let temp_dir = tempfile::tempdir().expect("Able to create a temp dir");

//         // Tries to load the dir
//         let save: Result<BoardSave, PreviewLoadError> = load_save(temp_dir.path());
//         let error = save.expect_err("Must error");

//         assert_eq!(error, PreviewLoadError::CannotRead)
//     }

//     #[test]
//     fn load_save_pass() {
//         let temp_dir = tempfile::tempdir().expect("Able to create a temp dir");

//         // Tries to load the dir
//         let save = load_save(temp_dir.path());
//     }
// }

// #[derive(thiserror::Error, Debug)]
// #[cfg_attr(test, derive(PartialEq))]
// pub enum PreviewLoadError {
//     #[error("Unable to load file")]
//     CannotRead,
// }

// pub fn load_save<'a>(save_location: &'a Path) -> Result<BoardSave, PreviewLoadError> {
//     Err(PreviewLoadError::CannotRead)
// }
