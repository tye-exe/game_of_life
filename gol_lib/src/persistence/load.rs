use std::{fs::File, io, path::Path};

use serde::de::DeserializeOwned;

use super::{
    SimulationBlueprint, SimulationSave,
    preview::{BlueprintPreview, SavePreview},
};

/// Returns true if the given number is equal to the given usize.
fn usize_eq<Num>(unsigned_size: usize, other_num: Num) -> bool
where
    Num: TryFrom<usize> + PartialEq,
{
    let Ok(converted): Result<Num, _> = unsigned_size.try_into() else {
        return false;
    };

    converted == other_num
}

/// The possible errors when attempting to parse a save file from disk.
#[derive(thiserror::Error, Debug)]
pub enum SaveParseError {
    #[error("Unable to read file")]
    FileRead(#[from] std::io::Error),
    #[error("File is not a valid save file")]
    InvalidData(#[from] serde_json::Error),
    #[error(
        "Simulation data does not match the area allocated for this board. 'data: {data_area}' : 'allocated: {allocated_area}'"
    )]
    UnexpectedSize {
        data_area: usize,
        allocated_area: u128,
    },
}

/// The possible errors when attempting to parse a blueprint from disk.
#[derive(thiserror::Error, Debug)]
pub enum BlueprintParseError {
    #[error("Unable to read file")]
    FileRead(#[from] std::io::Error),
    #[error("File is not a valid blueprint file")]
    InvalidData(#[from] serde_json::Error),
    #[error(
        "Blueprint cell data does not match the area allocated for this blueprint. 'data: {data_size}' : 'allocated: {allocated_size}'"
    )]
    UnexpectedSize {
        data_size: usize,
        allocated_size: u128,
    },
    #[error("The size of this blueprint is too large for your computer.")]
    BlueprintTooBig,
}

/// Attempts to parse the board data from a save at the given file path.
pub fn load_board_data(save_location: impl AsRef<Path>) -> Result<SimulationSave, SaveParseError> {
    let file = std::fs::File::open(save_location)?;
    let simulation: SimulationSave = serde_json::from_reader(file)?;

    let area: u128 =
        simulation.board_area.x_difference() as u128 * simulation.board_area.y_difference() as u128;
    let unsigned_size = simulation.board_data.len();

    if !usize_eq(unsigned_size, area) {
        return Err(SaveParseError::UnexpectedSize {
            data_area: unsigned_size,
            allocated_area: area,
        });
    }

    Ok(simulation)
}

/// Attempts to parse a blueprint from the file at the given path.
pub fn load_blueprint(
    blueprint_location: impl AsRef<Path>,
) -> Result<SimulationBlueprint, BlueprintParseError> {
    let file = std::fs::File::open(blueprint_location)?;
    let blueprint: SimulationBlueprint = serde_json::from_reader(file)?;

    let area = blueprint.x_size as u128 * blueprint.y_size as u128;
    let unsigned_size = blueprint.blueprint_data.len();

    if !usize_eq(unsigned_size, area) {
        return Err(BlueprintParseError::UnexpectedSize {
            data_size: unsigned_size,
            allocated_size: area,
        });
    }

    Ok(blueprint)
}

/// Finds and parses [`SavePreview`]s from the given directory.
pub fn load_save_preview(
    save_location: impl AsRef<Path>,
) -> Result<Box<[Result<SavePreview, ParseError>]>, io::Error> {
    load(save_location)
}

/// Finds and parses [`BlueprintPreview`]s from the given directory.
pub fn load_blueprint_preview(
    blueprint_location: impl AsRef<Path>,
) -> Result<Box<[Result<BlueprintPreview, ParseError>]>, io::Error> {
    load(blueprint_location)
}

/// Finds and parses [`Preview`]s from the given directory.
pub fn load_preview<T: DeserializeOwned>(
    preview_location: impl AsRef<Path>,
) -> Result<Box<[Result<T, ParseError>]>, io::Error> {
    load(preview_location)
}

/// The errors that can occur when attempting to parse data from a file.
#[derive(thiserror::Error, Debug)]
#[cfg_attr(test, derive(kinded::Kinded))]
pub enum ParseError {
    /// Unable to read file.
    #[error("Unable to read file: {0}")]
    FileParse(#[from] std::io::Error),
    /// The file contains invalid data.
    #[error("File '{path:?}' is not a valid data file: {serde_error}")]
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
    directory: impl AsRef<Path>,
) -> Result<Box<[Result<Data, ParseError>]>, std::io::Error> {
    let parsed_data = std::fs::read_dir(directory)?
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

#[cfg(test)]
mod tests {
    use std::time::UNIX_EPOCH;

    use bitvec::vec::BitVec;
    use tempfile::tempdir;

    use crate::{
        Area,
        persistence::{
            SaveBuilder, SimulationBlueprint, SimulationSave,
            load::{BlueprintParseError, SaveParseError},
        },
    };

    use super::load_board_data;

    #[test]
    fn valid_blueprint() {
        let temp_dir = tempdir().expect("Able to create temp dir");

        let name = "abc";
        let description = "efg";
        let tags = Box::new(["hi"]);
        let time = UNIX_EPOCH;

        let path = SaveBuilder::new_blueprint(SimulationBlueprint {
            x_size: 1,
            y_size: 3,
            blueprint_data: {
                let mut bit_vec = BitVec::new();
                bit_vec.push(true);
                bit_vec.push(true);
                bit_vec.push(true);
                bit_vec.into_boxed_bitslice()
            },
        })
        .name(name)
        .desciprtion(description)
        .tags(tags)
        .time(time)
        .save(temp_dir.path())
        .expect("Able to save blueprint");

        let simulation_blueprint = super::load_blueprint(path).expect("Can load blueprint");

        assert_eq!(
            simulation_blueprint,
            SimulationBlueprint {
                x_size: 1,
                y_size: 3,
                blueprint_data: {
                    let mut bit_vec = BitVec::new();
                    bit_vec.push(true);
                    bit_vec.push(true);
                    bit_vec.push(true);
                    bit_vec.into_boxed_bitslice()
                }
            },
            "The parsed blueprint does not match the expected blueprint."
        )
    }

    #[test]
    fn bad_blueprint() {
        let temp_dir = tempdir().expect("Unable to create tempoary directory.");

        let path = SaveBuilder::new_blueprint(SimulationBlueprint {
            x_size: 2,
            y_size: 2,
            blueprint_data: {
                let mut bit_vec = BitVec::new();
                bit_vec.push(true);
                bit_vec.into_boxed_bitslice()
            },
        })
        .save(temp_dir.path())
        .expect("Can save blueprint");

        assert!(
            matches!(
                super::load_blueprint(path),
                Err(BlueprintParseError::UnexpectedSize { .. })
            ),
            "The blueprint must fail to parse as the data size does not match the allocated size."
        );
    }

    #[test]
    fn valid_board() {
        let temp_dir = tempdir().expect("Able to create temp dir");

        let path = SaveBuilder::new_save(SimulationSave {
            generation: 0,
            board_area: Area::new((0, 0), (2, 2)),
            board_data: {
                let mut bit_vec = BitVec::new();
                bit_vec.push(true);
                bit_vec.push(true);
                bit_vec.push(true);
                bit_vec.push(true);
                bit_vec.into_boxed_bitslice()
            },
        })
        .save(temp_dir.path())
        .expect("Able to save file");

        let simulation_save = load_board_data(path).expect("Valid board data");

        assert_eq!(
            simulation_save,
            SimulationSave {
                generation: 0,
                board_area: Area::new((0, 0), (2, 2)),
                board_data: {
                    let mut bit_vec = BitVec::new();
                    bit_vec.push(true);
                    bit_vec.push(true);
                    bit_vec.push(true);
                    bit_vec.push(true);
                    bit_vec.into_boxed_bitslice()
                },
            },
            "The parsed save does not match the expected save data."
        )
    }

    #[test]
    fn invalid_board() {
        let temp_dir = tempdir().expect("Able to create temp dir");

        let path = SaveBuilder::new_save(SimulationSave {
            generation: 0,
            board_area: Area::new((0, 0), (2, 1)),
            board_data: {
                let mut bit_vec = BitVec::new();
                bit_vec.push(true);
                bit_vec.push(true);
                bit_vec.push(true);
                bit_vec.push(true);
                bit_vec.into_boxed_bitslice()
            },
        })
        .save(temp_dir.path())
        .expect("Able to save file");

        assert!(
            matches!(
                load_board_data(path),
                Err(SaveParseError::UnexpectedSize { .. })
            ),
            "Save must fail to parse as the data size does not match the allocated size."
        )
    }
}
