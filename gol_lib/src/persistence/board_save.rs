use crate::{persistence::SimulationSave, GlobalPosition};
use std::fs::File;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{SaveData, CURRENT_SAVE_VERSION};

/// The possible errors when saving a board save.
#[derive(thiserror::Error, Debug)]
pub enum BoardSaveError {
    /// The save content cannot be converted into the save file format.
    #[error("Unable to convert save data into file.")]
    SaveFormat,
    /// The save file already exists.
    #[error("This save already exists.")]
    FileOpen(std::io::Error),
    /// Unable to write the save file to disk.
    #[error("Unable to write file.")]
    WriteFail(#[from] std::io::Error),
}

/// Builder for easily creating a save.
#[cfg_attr(any(test), derive(Debug, PartialEq))]
pub struct SaveBuilder {
    save_name: Option<Box<str>>,
    save_description: Option<Box<str>>,
    save_time: Option<SystemTime>,
    view_position: Option<GlobalPosition>,

    simulation_save: SimulationSave,
}

impl SaveBuilder {
    /// Creates a new save builder with no values set.
    pub fn new(simulation_save: SimulationSave) -> Self {
        Self {
            simulation_save,
            save_name: None,
            save_description: None,
            save_time: None,
            view_position: None,
        }
    }

    /// The name of the save. This is not the filename.
    pub fn name(mut self, name: impl Into<Box<str>>) -> Self {
        self.save_name = Some(name.into());
        self
    }

    /// A description of the save.
    pub fn desciprtion(mut self, description: impl Into<Box<str>>) -> Self {
        self.save_description = Some(description.into());
        self
    }

    /// The view position of the save.
    pub fn view_position(mut self, view_position: GlobalPosition) -> Self {
        self.view_position = Some(view_position);
        self
    }

    /// The time the save was created.
    pub fn time(mut self, time: SystemTime) -> Self {
        self.save_time = Some(time);
        self
    }
}

impl SaveBuilder {
    /// Saves the board at the given save path.
    /// The save path should be the the path to the save location, **without** the filename or extension, as these will be added during the method.
    ///
    /// The returned value is the file path to the saved file, including the filename. Or an error if one occurred.
    pub fn save(self, save_path: impl Into<PathBuf>) -> Result<Box<Path>, BoardSaveError> {
        let SaveBuilder {
            save_name,
            save_description,
            save_time,
            view_position,
            simulation_save,
        } = self;

        let save_path = save_path.into();
        let save_name = save_name.unwrap_or("".into());
        let save_description = save_description.unwrap_or("".into());
        let mut save_path: PathBuf = save_path.into();

        // Use time to differentiate saves with the same name.
        let save_time = save_time
            .unwrap_or_else(|| SystemTime::now())
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::default());

        // Generate file name from save content.
        let file_name = {
            // Don't hash board data as it might be very large.
            let mut hasher = DefaultHasher::new();

            save_name.hash(&mut hasher);
            save_description.hash(&mut hasher);
            simulation_save.board_area.hash(&mut hasher);
            save_time.hash(&mut hasher);

            hasher.finish().to_string()
        };

        // Need to push to create new file.
        save_path.push(file_name);
        save_path.set_extension("save");

        let data = SaveData {
            version: CURRENT_SAVE_VERSION,
            save_name,
            save_description,
            save_time,
            view_position,
            simulation_save,
        };

        // Conversion into string can fail somehow?
        let file_data = serde_json::to_string(&data).map_err(|_| BoardSaveError::SaveFormat)?;

        // Write file if it doesn't exist.
        File::create_new(&save_path)
            .map_err(|err| BoardSaveError::FileOpen(err))?
            .write_all(&file_data.into_bytes())?;

        Ok(save_path.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generate_save_name(
        path: &Path,
        save_name: &str,
        save_description: &str,
        save_time: SystemTime,
        simulation: &SimulationSave,
    ) -> PathBuf {
        let mut save_path = path.to_path_buf();

        // Use time to differentiate saves with the same name.
        let save_time = save_time
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::default());

        // Generate file name from save content.
        let file_name = {
            // Don't hash board data as it might be very large.
            let mut hasher = DefaultHasher::new();

            save_name.hash(&mut hasher);
            save_description.hash(&mut hasher);
            simulation.board_area.hash(&mut hasher);
            save_time.hash(&mut hasher);

            hasher.finish().to_string()
        };

        save_path.push(file_name);
        save_path.set_extension("save");
        save_path
    }

    #[test]
    fn can_save_board() {
        let temp_dir = tempfile::tempdir().expect("Able to create a temp dir");
        let save_name = "save";
        let save_description = "description";
        let save_time = SystemTime::now();

        SaveBuilder::new(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time)
            .save(temp_dir.path())
            .expect("Can save file");
    }

    #[test]
    fn save_board_name() {
        let temp_dir = tempfile::tempdir().expect("Able to create a temp dir");
        let save_name = "save";
        let save_description = "description";
        let save_time = SystemTime::now();

        // Generate correct name.
        let save_path = generate_save_name(
            temp_dir.path(),
            save_name,
            save_description,
            save_time,
            &SimulationSave::default(),
        );

        // Call method
        let save_board = SaveBuilder::new(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time)
            .save(temp_dir.path())
            .expect("Can save file");

        assert_eq!(save_board, save_path.into_boxed_path());
    }

    #[test]
    fn save_board_file_exists() {
        let temp_dir = tempfile::tempdir().expect("Able to create a temp dir");

        let save_name = "save";
        let save_description = "description";
        let save_time = SystemTime::now();

        let save_path = generate_save_name(
            temp_dir.path(),
            save_name,
            save_description,
            save_time,
            &Default::default(),
        );

        // Write file with same name
        std::fs::write(save_path, "").expect("Can write file");

        // Call method
        let save_board = SaveBuilder::new(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time)
            .save(temp_dir.path())
            .expect_err("Must error as file exists");

        assert!(match save_board {
            BoardSaveError::FileOpen(..) => {
                true
            }
            BoardSaveError::SaveFormat => {
                false
            }
            BoardSaveError::WriteFail(..) => {
                false
            }
        });
    }
}
