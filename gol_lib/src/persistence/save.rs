use crate::{GlobalPosition, persistence::SimulationSave};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use super::{CURRENT_SAVE_VERSION, SaveData, generate_filename};

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
    name: Option<Box<str>>,
    description: Option<Box<str>>,
    tags: Option<Box<[Box<str>]>>,

    time: Option<SystemTime>,
    view_position: Option<GlobalPosition>,

    simulation_save: SimulationSave,
}

impl SaveBuilder {
    /// Creates a new save builder with no values set.
    pub fn new(simulation_save: SimulationSave) -> Self {
        Self {
            simulation_save,
            name: None,
            description: None,
            time: None,
            view_position: None,
            tags: None,
        }
    }

    /// The name of the save. This is not the filename.
    pub fn name(mut self, name: impl Into<Box<str>>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// A description of the save.
    pub fn desciprtion(mut self, description: impl Into<Box<str>>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// The view position of the save.
    pub fn view_position(mut self, view_position: GlobalPosition) -> Self {
        self.view_position = Some(view_position);
        self
    }

    /// The time the save was created.
    pub fn time(mut self, time: SystemTime) -> Self {
        self.time = Some(time);
        self
    }

    /// The tags this save belongs to.
    pub fn tags(mut self, tags: Box<[Box<str>]>) -> Self {
        self.tags = Some(tags);
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
            name: save_name,
            description: save_description,
            time: save_time,
            tags: save_tags,
            view_position,
            simulation_save,
        } = self;

        let mut save_path: PathBuf = save_path.into();
        let save_name = save_name.unwrap_or("".into());
        let save_description = save_description.unwrap_or("".into());
        let save_tags = save_tags.unwrap_or(Box::new([]));

        // Use time to differentiate saves with the same name.
        let save_time = save_time
            .unwrap_or_else(|| SystemTime::now())
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::default());

        let file_name = generate_filename(
            &simulation_save.board_area,
            &save_name,
            &save_description,
            &save_tags,
            &save_time,
        );

        // Need to push to create new file.
        save_path.push(file_name);

        let data = SaveData {
            version: CURRENT_SAVE_VERSION,
            name: save_name,
            description: save_description,
            time: save_time,
            view_position,
            simulation_save,
            tags: save_tags,
        };

        // Conversion into string can fail somehow?
        let file_data = serde_json::to_string(&data).map_err(|_| BoardSaveError::SaveFormat)?;

        // Write file if it doesn't exist.
        File::create_new(&save_path)
            .map_err(|err| BoardSaveError::FileOpen(err))?
            .write_all(&file_data.into_bytes())?;

        Ok(save_path.into())
    }

    /// Generates the save path that the current data will be saved at if [`Self::save`] was called.
    ///
    /// **ONLY AVAILABLE WHEN RUNNING TESTS**
    #[cfg(test)]
    pub(crate) fn generate_save_name(&self, path: &Path) -> std::path::PathBuf {
        let mut save_path: PathBuf = path.into();
        let save_name = self.name.clone().unwrap_or("".into());
        let save_description = self.description.clone().unwrap_or("".into());
        let save_tags = self.tags.clone().unwrap_or(Box::new([]));

        // Use time to differentiate saves with the same name.
        let save_time = self
            .time
            .unwrap_or_else(|| SystemTime::now())
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::default());

        let file_name = generate_filename(
            &self.simulation_save.board_area,
            &save_name,
            &save_description,
            &save_tags,
            &save_time,
        );

        save_path.push(file_name);
        save_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// The generated save name will be correct.
    fn name_generates_correctly() {
        let temp_dir = tempfile::tempdir().expect("Able to create a temp dir");
        let save_name = "save";
        let save_description = "description";
        let save_tags = Box::new(["test".to_owned().into_boxed_str()]);

        // Use unix epoch for consistency
        let save_time = SystemTime::UNIX_EPOCH;

        let generate_save_name = SaveBuilder::new(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time)
            .tags(save_tags)
            .generate_save_name(temp_dir.path());

        assert!(generate_save_name.ends_with("9011655623179715335.save"));
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
    /// The test name generation matches the actual name generation.
    fn save_board_name() {
        let temp_dir = tempfile::tempdir().expect("Able to create a temp dir");
        let save_name = "save";
        let save_description = "description";
        let save_time = SystemTime::now();

        let save_builder = SaveBuilder::new(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time);

        // Generate correct name.
        let save_path = save_builder.generate_save_name(temp_dir.path());

        // Call method
        let save_board = save_builder.save(temp_dir.path()).expect("Can save file");

        assert_eq!(save_board, save_path.into_boxed_path());
    }

    #[test]
    fn save_board_file_exists() {
        let temp_dir = tempfile::tempdir().expect("Able to create a temp dir");

        let save_name = "save";
        let save_description = "description";
        let save_time = SystemTime::now();

        let save_builder = SaveBuilder::new(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time);

        let save_path = save_builder.generate_save_name(temp_dir.path());

        // Write file with same name
        std::fs::write(save_path, "").expect("Can write file");

        // Try save
        let save_board = save_builder
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
