use std::{fs::File, path::Path};

use serde::de::DeserializeOwned;

use super::SimulationSave;

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
pub(crate) fn load<'a, Data: DeserializeOwned>(
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
