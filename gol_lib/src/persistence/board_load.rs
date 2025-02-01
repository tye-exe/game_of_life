use std::path::Path;

use super::SaveData;

/// The possible errors when attempting to parse a save file from disk.
#[derive(thiserror::Error, Debug)]
pub enum SaveParseError {
    #[error("Unable to read file")]
    FileRead(#[from] std::io::Error),
    #[error("File is not a valid save file")]
    InvalidData(#[from] serde_json::Error),
}

/// Attempts to parse a save file from disk at the given path.
pub fn load_save<'a>(save_location: impl Into<&'a Path>) -> Result<SaveData, SaveParseError> {
    let file = std::fs::File::open(save_location.into())?;
    let save = serde_json::from_reader(file)?;
    Ok(save)
}
