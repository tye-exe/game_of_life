use std::{path::Path, time::Duration};

use super::{load, ParseError};

/// Finds and parses [`SavePreview`]s from the given directory.
pub fn load_preview<'a>(
    save_location: impl Into<&'a Path>,
) -> Result<Box<[Result<SavePreview, ParseError>]>, std::io::Error> {
    load(save_location)
}

/// Contains the information about a board save, without actually containing the board save data.
/// This is useful to load in as a preview for a save, without having to load the entire board into memory.
#[cfg_attr(test, derive(Debug, PartialEq))]
#[derive(serde::Deserialize)]
pub struct SavePreview {
    /// The save file version.
    version: u16,

    /// The name of the save. This is not the name of the save file.
    name: Box<str>,
    /// A description of the save.
    description: Box<str>,
    /// The generation this save was made on.
    generation: u64,
    /// The time the save was made.
    time: Duration,

    /// The tags this save has.
    tags: Box<[Box<str>]>,
}

impl SavePreview {
    /// The save file version of the save file.
    pub fn get_version(&self) -> u16 {
        self.version
    }

    /// The name of the save. This is not the name of the save file.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// The description for the save.
    pub fn get_description(&self) -> &str {
        &self.description
    }

    /// The generation the save was made on.
    pub fn get_generation(&self) -> u64 {
        self.generation
    }

    /// The time the save was made.
    pub fn get_time(&self) -> Duration {
        self.time
    }

    /// The tags this save is part of.
    pub fn get_tags(&self) -> &[Box<str>] {
        &self.tags
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::persistence::{board_save::SaveBuilder, ParseErrorKind, CURRENT_SAVE_VERSION};

    use super::*;

    #[test]
    /// An empty dir must return an empty slice.
    fn empty_dir() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");

        let parse_saves = load_preview(temp_dir.path());
        assert!(parse_saves.unwrap().is_empty());
    }

    #[test]
    /// An invalid save should be parsed as an error.
    fn invalid_save() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");

        // Write invalid file
        let mut path_buf = temp_dir.path().to_path_buf();
        path_buf.push("Invalid");
        std::fs::write(path_buf, "Invalid!!!").expect("Able to write file");

        let parse_saves = load_preview(temp_dir.path()).unwrap();
        assert_eq!(parse_saves.len(), 1);

        // Must return with invalid data error
        let save_error = parse_saves.get(0).unwrap().as_ref().unwrap_err();
        assert_eq!(save_error.kind(), ParseErrorKind::InvalidData)
    }

    #[test]
    /// A valid save should parse correctly
    fn valid_save() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");
        let save_name = "name";
        let save_description = "description";
        let save_tags = Box::new(["test".to_owned().into_boxed_str()]);
        let save_time = SystemTime::now();

        let path = SaveBuilder::new(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time)
            .tags(save_tags.clone())
            .save(temp_dir.path())
            .expect("Can save file");

        let parse_saves = load_preview(temp_dir.path()).unwrap();
        assert_eq!(parse_saves.len(), 1);

        assert_eq!(
            parse_saves.get(0).unwrap().as_ref().unwrap(),
            &SavePreview {
                version: CURRENT_SAVE_VERSION,
                name: save_name.into(),
                description: save_description.into(),
                generation: 0,
                time: save_time
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::default()),
                tags: save_tags
            }
        );
    }

    #[test]
    /// Tests parsing both a valid save file and an invalid save file.
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
        let path = SaveBuilder::new(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time)
            .tags(save_tags.clone())
            .save(temp_dir.path())
            .expect("Can save file");

        let parse_saves = load_preview(temp_dir.path()).unwrap();
        assert_eq!(parse_saves.len(), 2);

        // Get "correct" saves
        let save_0 = parse_saves.get(0).unwrap().as_ref();
        let save_1 = parse_saves.get(1).unwrap().as_ref();

        let invalid = {
            if save_0.is_err() {
                save_0
            } else {
                save_1
            }
        };

        let valid = {
            if save_0.is_err() {
                save_1
            } else {
                save_0
            }
        };

        assert_eq!(invalid.unwrap_err().kind(), ParseErrorKind::InvalidData);

        assert_eq!(
            valid.unwrap(),
            &SavePreview {
                version: CURRENT_SAVE_VERSION,
                name: save_name.into(),
                description: save_description.into(),
                generation: 0,
                time: save_time
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::default()),
                tags: save_tags
            }
        );
    }

    #[test]
    /// A file with invalid data must return the file path of the invalid file.
    fn invalid_returns_path() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");

        // Write invalid file
        let mut path_buf = temp_dir.path().to_path_buf();
        path_buf.push("Invalid");
        std::fs::write(path_buf.clone(), "Invalid!!!").expect("Able to write file");

        let parse_saves = load_preview(temp_dir.path()).unwrap();
        assert_eq!(parse_saves.len(), 1);

        // Must return with invalid data error
        let save_error = parse_saves.get(0).unwrap().as_ref().unwrap_err();
        assert_eq!(save_error.file_path(), Some(path_buf).as_deref());
        assert_eq!(save_error.kind(), ParseErrorKind::InvalidData)
    }
}
