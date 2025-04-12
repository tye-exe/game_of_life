use super::{Blueprint, Save};
use crate::Area;
use std::time::Duration;

/// The metadata about a board save.
pub type SavePreview = Preview<SavePreviewData>;
/// The metadata about a blueprint.
pub type BlueprintPreview = Preview<BlueprintPreviewData>;

/// The unique preview data for a save file.
#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(serde::Deserialize, Clone)]
pub struct SavePreviewData {
    /// The area the save takes up on the board.
    board_area: Area,
    /// The generation this save was made on.
    generation: u64,
}

/// The unique preview data for a blueprint file.
#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(serde::Deserialize, Clone)]
pub struct BlueprintPreviewData {
    /// The x size of this blueprint.
    x_size: u32,
    /// The y size of this blueprint.
    y_size: u32,
}

/// Contains the information about a save or blueprint, without actually containing the board data.
/// This is useful to load in as a preview, because the (potentially large) board data will not be loaded.
#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(serde::Deserialize, Clone)]
pub struct Preview<Data> {
    /// The save file version.
    version: u16,

    /// The name of the save. This is not the name of the save file.
    name: Box<str>,
    /// A description of the save.
    description: Box<str>,
    /// The time the save was made.
    time: Duration,
    /// The tags this save has.
    tags: Box<[Box<str>]>,

    /// The data unique to the preview type.
    #[serde(flatten)]
    data: Data,
}

impl<Data> Preview<Data> {
    /// The save file version of the file.
    pub fn get_version(&self) -> u16 {
        self.version
    }

    /// The name of the data. This is not the filename of the file.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// The description for the data.
    pub fn get_description(&self) -> &str {
        &self.description
    }

    /// The time the save of the data was made.
    pub fn get_time(&self) -> Duration {
        self.time
    }

    /// The tags this save data is part of.
    pub fn get_tags(&self) -> &[Box<str>] {
        &self.tags
    }
}

impl Preview<SavePreviewData> {
    /// The generation the save was made on.
    pub fn get_generation(&self) -> u64 {
        self.data.generation
    }

    /// The area the save takes up on the board.
    pub fn get_board_area(&self) -> Area {
        self.data.board_area
    }

    /// The filename of the save file (including the extension).
    pub fn get_filename(&self) -> String {
        Save::generate_filename(
            self.data.board_area,
            &self.name,
            &self.description,
            &self.tags,
            &self.time,
        )
    }
}

impl Preview<BlueprintPreviewData> {
    /// The x size of this blueprint.
    pub fn get_x_size(&self) -> u32 {
        self.data.x_size
    }

    /// The y size of this blueprint.
    pub fn get_y_size(&self) -> u32 {
        self.data.y_size
    }

    /// The filename of the blueprint file (including the extension).
    pub fn get_filename(&self) -> String {
        Blueprint::generate_filename(
            self.data.x_size,
            self.data.y_size,
            &self.name,
            &self.description,
            &self.tags,
            &self.time,
        )
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::persistence::{
        CURRENT_SAVE_VERSION, SaveBuilder, load::ParseErrorKind, load_blueprint_preview,
        load_save_preview,
    };

    use super::*;

    /// An empty dir must return an empty slice.
    #[test]
    fn empty_dir() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");

        let parse_saves = load_save_preview(temp_dir.path());
        assert!(parse_saves.unwrap().is_empty());
    }

    /// An invalid save should be parsed as an error.
    #[test]
    fn invalid_save() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");

        // Write invalid file
        let mut path_buf = temp_dir.path().to_path_buf();
        path_buf.push("Invalid");
        std::fs::write(path_buf, "Invalid!!!").expect("Able to write file");

        let parse_saves = load_save_preview(temp_dir.path()).unwrap();
        assert_eq!(parse_saves.len(), 1);

        // Must return with invalid data error
        let save_error = parse_saves.get(0).unwrap().as_ref().unwrap_err();
        assert_eq!(save_error.kind(), ParseErrorKind::InvalidData)
    }

    /// A valid save should parse correctly
    #[test]
    fn valid_save() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");
        let save_name = "name";
        let save_description = "description";
        let save_tags = Box::new(["test".to_owned().into_boxed_str()]);
        let save_time = SystemTime::now();

        let _ = SaveBuilder::new_save(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time)
            .tags(save_tags.clone())
            .save(temp_dir.path())
            .expect("Can save file");

        let parse_saves = load_save_preview(temp_dir.path()).unwrap();
        assert_eq!(parse_saves.len(), 1);

        assert_eq!(
            parse_saves.get(0).unwrap().as_ref().unwrap(),
            &Preview {
                version: CURRENT_SAVE_VERSION,
                name: save_name.into(),
                description: save_description.into(),
                time: save_time
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::default()),
                tags: save_tags,
                data: SavePreviewData {
                    board_area: Default::default(),
                    generation: 0
                }
            }
        );
    }

    /// Tests parsing both a valid save file and an invalid save file.
    #[test]
    fn parse_mix() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");
        let save_name = "name";
        let save_description = "description";
        let save_time = SystemTime::now();
        let save_tags = Box::new(["test".to_owned().into_boxed_str()]);

        // Write invalid file
        let mut path_buf = temp_dir.path().to_path_buf();
        path_buf.push("Invalid");
        std::fs::write(path_buf, "Invalid!!!").expect("Able to write file");

        let mut path = temp_dir.path().to_path_buf();
        path.push("sub_dir");
        std::fs::create_dir(&path).expect("Can create subdir");

        // Write valid file
        let _ = SaveBuilder::new_save(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time)
            .tags(save_tags.clone())
            .save(temp_dir.path())
            .expect("Can save file");

        let parse_saves = load_save_preview(temp_dir.path()).unwrap();
        assert_eq!(parse_saves.len(), 2);

        // Get "correct" saves
        let save_0 = parse_saves.get(0).unwrap().as_ref();
        let save_1 = parse_saves.get(1).unwrap().as_ref();

        let invalid = { if save_0.is_err() { save_0 } else { save_1 } };

        let valid = { if save_0.is_err() { save_1 } else { save_0 } };

        assert_eq!(invalid.unwrap_err().kind(), ParseErrorKind::InvalidData);

        assert_eq!(
            valid.unwrap(),
            &Preview {
                version: CURRENT_SAVE_VERSION,
                name: save_name.into(),
                description: save_description.into(),
                time: save_time
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::default()),
                tags: save_tags,
                data: SavePreviewData {
                    board_area: Default::default(),
                    generation: 0
                }
            }
        );
    }

    /// A file with invalid data must return the file path of the invalid file.
    #[test]
    fn invalid_returns_path() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");

        // Write invalid file
        let mut path_buf = temp_dir.path().to_path_buf();
        path_buf.push("Invalid");
        std::fs::write(path_buf.clone(), "Invalid!!!").expect("Able to write file");

        let parse_saves = load_save_preview(temp_dir.path()).unwrap();
        assert_eq!(parse_saves.len(), 1);

        // Must return with invalid data error
        let save_error = parse_saves.get(0).unwrap().as_ref().unwrap_err();
        assert_eq!(save_error.file_path(), Some(path_buf).as_deref());
        assert_eq!(save_error.kind(), ParseErrorKind::InvalidData)
    }

    /// The filename returned by the preview is the correct filename.
    #[test]
    fn get_save_path() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");
        let save_name = "name";
        let save_description = "description";
        let save_time = SystemTime::now();
        let save_tags = Box::new(["test".to_owned().into_boxed_str()]);

        // Creates save file.
        let path = SaveBuilder::new_save(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time)
            .tags(save_tags)
            .save(temp_dir.path())
            .expect("Able to write save file");

        // Temp binding to satisfy rust lifetimes
        let binding =
            load_save_preview(temp_dir.path()).expect("Can read from tempoary save directory");

        // Gets the parsed preview.
        let save_preview = binding
            .get(0)
            .expect("One save file will be parsed")
            .as_ref()
            .expect("Save file is valid");

        // Gets the filename from the file creation.
        let filename = path
            .file_name()
            .expect("The returned save file path is valid.")
            .to_str()
            .expect("The filename will be valid unicode");

        assert_eq!(filename, save_preview.get_filename());
    }

    /// A valid blueprint should parse correctly
    #[test]
    fn valid_blueprint() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");
        let save_name = "name";
        let save_description = "description";
        let save_tags = Box::new(["test".to_owned().into_boxed_str()]);
        let save_time = SystemTime::now();

        let _ = SaveBuilder::new_blueprint(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time)
            .tags(save_tags.clone())
            .save(temp_dir.path())
            .expect("Can save file");

        let parse_saves = load_blueprint_preview(temp_dir.path()).unwrap();
        assert_eq!(parse_saves.len(), 1);

        assert_eq!(
            parse_saves.get(0).unwrap().as_ref().unwrap(),
            &Preview {
                version: CURRENT_SAVE_VERSION,
                name: save_name.into(),
                description: save_description.into(),
                time: save_time
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::default()),
                tags: save_tags,
                data: BlueprintPreviewData {
                    x_size: 0,
                    y_size: 0
                }
            }
        );
    }

    /// The actual filename and the generated filename are correct.
    #[test]
    fn blueprint_filename() {
        const EXPECTED_FILENAME: &str = "18105438302258333504.save";

        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");
        let save_name = "name";
        let save_description = "description";
        let save_tags = Box::new(["test".to_owned().into_boxed_str()]);
        // For consistency
        let save_time = UNIX_EPOCH;

        // Name of file on disk.
        let file_path = SaveBuilder::new_blueprint(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time)
            .tags(save_tags.clone())
            .save(temp_dir.path())
            .expect("Can save file");

        let file_name = file_path
            .file_name()
            .expect("Path will terminate in valid character")
            .to_str()
            .expect("The filename will only contain valid characters");

        assert_eq!(
            file_name, EXPECTED_FILENAME,
            "The filename of a blueprint file will match the expected filename."
        );

        // Name of test generated filename.
        let generated_path = SaveBuilder::new_blueprint(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time)
            .tags(save_tags.clone())
            .generate_filename(temp_dir.path());

        let generated_name = generated_path
            .file_name()
            .expect("Path will terminate in valid character")
            .to_str()
            .expect("The filename will only contain valid characters");

        assert_eq!(
            generated_name, EXPECTED_FILENAME,
            "The test generated filename of a blueprint will match the expected filename."
        );
    }
}
