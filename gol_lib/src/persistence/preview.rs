use crate::persistence::CURRENT_SAVE_VERSION;
use std::{path::Path, time::Duration};
use walkdir::WalkDir;

/// The errors that can occur when attempting to parse a [`SavePreview`] from a save file.
#[derive(thiserror::Error, Debug)]
#[cfg_attr(test, derive(kinded::Kinded))]
pub enum PreviewParseError {
    /// Encountered an error whilst traversing files in save location.
    #[error("Failed to parse possible save files: {0}")]
    FileSearch(#[from] walkdir::Error),
    /// Unable to read file.
    #[error("Unable to read save file: {error}")]
    FileParse {
        error: std::io::Error,
        path: Box<Path>,
    },
    /// The file is not a valid save file.
    #[error("File is not a valid save file: {error}")]
    InvalidData {
        error: serde_json::Error,
        path: Box<Path>,
    },
}

impl PreviewParseError {
    /// The path to the file that cause the parse error. Or `None` if the path cannot be determined.
    pub fn path(&self) -> Option<&Path> {
        match self {
            PreviewParseError::FileSearch(error) => error.path(),
            PreviewParseError::FileParse { path, .. } => Some(path),
            PreviewParseError::InvalidData { path, .. } => Some(path),
        }
    }
}

/// Finds and parses [`SavePreview`]s recursively from the given directory.
pub fn load_preview<'a>(
    save_location: impl Into<&'a Path>,
) -> Box<[Result<SavePreview, PreviewParseError>]> {
    WalkDir::new(save_location.into())
        .follow_links(true)
        .into_iter()
        // Only parse files
        .filter_map(|file| match file {
            Ok(file) if file.file_type().is_file() => Some(Ok(file)),
            Ok(_) => None,
            Err(err) => Some(Err(err.into())),
        })
        // Attempt to parse file
        .map(|file| match file {
            Ok(file) => SavePreview::new(file.path()),
            Err(err) => Err(err),
        })
        .collect()
}

/// Contains the information about a board save, without actually containing the board save data.
/// This is useful to load in as a preview for a save, without having to load the entire board into memory.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct SavePreview {
    /// The save file version.
    version: u16,

    /// The name of the save. This is not the name of the save file.
    save_name: Box<str>,
    /// A description of the save.
    save_description: Box<str>,
    /// The generation this save was made on.
    generation: u64,
    /// The time the save was made
    save_time: Duration,

    /// The path to the save file. This includes the filename.
    save_path: Box<Path>,
}

impl SavePreview {
    /// Parses a new instance of [`SavePreview`] from the given filepath.
    fn new<'a>(save_path: impl Into<&'a Path>) -> Result<SavePreview, PreviewParseError> {
        /// Used to parse the data for SaveData instead of manual implementation.
        #[derive(serde::Deserialize)]
        struct PartialData {
            save_name: Box<str>,
            save_description: Box<str>,
            generation: u64,
            save_time: Duration,
        }

        let save_path = save_path.into();

        // Parse the file data.
        let file_data =
            std::fs::read_to_string(save_path).map_err(|err| PreviewParseError::FileParse {
                error: err,
                path: save_path.into(),
            })?;

        let PartialData {
            save_name,
            save_description,
            generation,
            save_time,
        } = serde_json::from_str(&file_data).map_err(|err| PreviewParseError::InvalidData {
            error: err,
            path: save_path.into(),
        })?;

        // Construct the finial object.
        Ok(SavePreview {
            version: CURRENT_SAVE_VERSION,
            save_name,
            save_description,
            generation,
            save_path: save_path.into(),
            save_time,
        })
    }

    /// The save file version of the save file.
    pub fn get_version(&self) -> u16 {
        self.version
    }

    /// The name of the save. This is not the name of the save file.
    pub fn get_save_name(&self) -> &str {
        &self.save_name
    }

    /// The description for the save.
    pub fn get_save_description(&self) -> &str {
        &self.save_description
    }

    /// The generation the save was made on.
    pub fn get_generation(&self) -> u64 {
        self.generation
    }

    /// The path to the save file.
    pub fn get_save_path(&self) -> &Path {
        &self.save_path
    }

    /// The time the save was made.
    pub fn get_time(&self) -> Duration {
        self.save_time
    }
}

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::persistence::board_save::SaveBuilder;

    use super::*;

    #[test]
    /// An empty dir must return an empty slice.
    fn empty_dir() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");

        let parse_saves = load_preview(temp_dir.path());
        assert!(parse_saves.is_empty());
    }

    #[test]
    /// An invalid save should be parsed as an error.
    fn invalid_save() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");

        // Write invalid file
        let mut path_buf = temp_dir.path().to_path_buf();
        path_buf.push("Invalid");
        std::fs::write(path_buf, "Invalid!!!").expect("Able to write file");

        let parse_saves = load_preview(temp_dir.path());
        assert_eq!(parse_saves.len(), 1);

        // Must return with invalid data error
        let save_error = parse_saves.get(0).unwrap().as_ref().unwrap_err();
        assert_eq!(save_error.kind(), PreviewParseErrorKind::InvalidData)
    }

    #[test]
    fn invalid_in_sub_dir() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");
        let mut path_buf = temp_dir.path().to_path_buf();

        // Create sub-dir
        path_buf.push("sub_dir");
        std::fs::create_dir(&path_buf).expect("Able to make sub dir");

        // Create invalid file
        path_buf.push("Invalid");
        std::fs::write(path_buf, "Invalid!!!").expect("Able to write file");

        let parse_saves = load_preview(temp_dir.path());
        assert_eq!(parse_saves.len(), 1);

        // Must return with invalid data error
        let save_error = parse_saves.get(0).unwrap().as_ref().unwrap_err();
        assert_eq!(save_error.kind(), PreviewParseErrorKind::InvalidData)
    }

    #[test]
    /// A valid save should parse correctly
    fn valid_save() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");
        let save_name = "name";
        let save_description = "description";
        let save_time = SystemTime::now();

        let path = SaveBuilder::new(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time)
            .save(temp_dir.path())
            .expect("Can save file");

        let parse_saves = load_preview(temp_dir.path());
        assert_eq!(parse_saves.len(), 1);

        assert_eq!(
            parse_saves.get(0).unwrap().as_ref().unwrap(),
            &SavePreview {
                version: CURRENT_SAVE_VERSION,
                save_name: save_name.into(),
                save_description: save_description.into(),
                generation: 0,
                save_path: path,
                save_time: save_time
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::default()),
            }
        );
    }

    #[test]
    /// A valid save in a sub-dir should parse correctly
    fn valid_in_sub_dir() {
        let temp_dir = tempfile::tempdir().expect("Able to create temp dir");
        let save_name = "name";
        let save_description = "description";
        let save_time = SystemTime::now();

        let mut path = temp_dir.path().to_path_buf();
        path.push("sub_dir");
        std::fs::create_dir(&path).expect("Can create subdir");

        let path = SaveBuilder::new(Default::default())
            .name(save_name)
            .desciprtion(save_description)
            .time(save_time)
            .save(temp_dir.path())
            .expect("Can save file");

        let parse_saves = load_preview(temp_dir.path());
        assert_eq!(parse_saves.len(), 1);

        assert_eq!(
            parse_saves.get(0).unwrap().as_ref().unwrap(),
            &SavePreview {
                version: CURRENT_SAVE_VERSION,
                save_name: save_name.into(),
                save_description: save_description.into(),
                generation: 0,
                save_path: path,
                save_time: save_time
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::default()),
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
            .save(temp_dir.path())
            .expect("Can save file");

        let parse_saves = load_preview(temp_dir.path());
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

        assert_eq!(
            invalid.unwrap_err().kind(),
            PreviewParseErrorKind::InvalidData
        );

        assert_eq!(
            valid.unwrap(),
            &SavePreview {
                version: CURRENT_SAVE_VERSION,
                save_name: save_name.into(),
                save_description: save_description.into(),
                generation: 0,
                save_path: path,
                save_time: save_time
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or(Duration::default()),
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

        let parse_saves = load_preview(temp_dir.path());
        assert_eq!(parse_saves.len(), 1);

        // Must return with invalid data error
        let save_error = parse_saves.get(0).unwrap().as_ref().unwrap_err();
        assert_eq!(save_error.path(), Some(path_buf).as_deref());
        assert_eq!(save_error.kind(), PreviewParseErrorKind::InvalidData)
    }
}
