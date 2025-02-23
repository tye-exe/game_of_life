use std::path::Path;

use super::{SaveData, SimulationSave};

/// The possible errors when attempting to parse a save file from disk.
#[derive(thiserror::Error, Debug)]
pub enum SaveParseError {
    #[error("Unable to read file")]
    FileRead(#[from] std::io::Error),
    #[error("File is not a valid save file")]
    InvalidData(#[from] serde_json::Error),
}

/// Attempts to parse the board data from a save at the given file path.
pub fn load_board_data(save_location: &Path) -> Result<SimulationSave, SaveParseError> {
    let file = std::fs::File::open(save_location)?;
    Ok(serde_json::from_reader(file)?)
}
