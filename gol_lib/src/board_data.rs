//! Contains the data structures used for handling blueprint & save data.

use bitvec::boxed::BitBox;
use std::{
    error::Error,
    hash::{DefaultHasher, Hash, Hasher},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};
use walkdir::WalkDir;

use crate::{Area, GlobalPosition};

/// The board data that a simulation consists of.
#[derive(serde::Deserialize)]
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq))]
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

pub fn parse_saves(save_root: &Path) -> Result<Box<[SaveData]>, Box<dyn Error>> {
    let mut saves = Vec::new();
    for file in WalkDir::new(save_root).follow_links(true) {
        let file = file?;
        if !file.file_type().is_file() {
            continue;
        }

        saves.push(SaveData::new(file.path())?);
    }

    Ok(saves.into())
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub struct SaveData {
    version: u32,

    save_name: Box<str>,
    save_description: Box<str>,
    generation: u64,

    save_path: Box<Path>,
}

impl SaveData {
    fn new(path_to_save: &Path) -> Result<SaveData, Box<dyn Error>> {
        todo!()
    }

    pub fn get_version(&self) -> u64 {
        todo!()
    }

    pub fn get_save_name(&self) -> Box<str> {
        todo!()
    }

    pub fn get_save_description(&self) -> Box<str> {
        todo!()
    }

    pub fn get_generation(&self) -> u64 {
        todo!()
    }

    pub fn get_save_path(&self) -> Box<Path> {
        todo!()
    }
}

/// The data that a save of a simulation consists of.
#[derive(serde::Serialize, serde::Deserialize)]
#[cfg_attr(any(test, debug_assertions), derive(Debug, PartialEq))]
pub struct BoardSave {
    version: u32,

    save_name: Box<str>,
    save_description: Box<str>,
    view_position: Option<GlobalPosition>,

    generation: u64,
    board_area: Area,
    board_data: BitBox,
}

impl BoardSave {
    pub fn new(
        save_name: impl Into<Box<str>>,
        save_description: impl Into<Box<str>>,
        view_position: Option<GlobalPosition>,
        simulation_save: SimulationSave,
    ) -> Self {
        Self {
            version: 0,
            save_name: save_name.into(),
            save_description: save_description.into(),
            view_position,
            generation: simulation_save.generation,
            board_area: simulation_save.board_area,
            board_data: simulation_save.board_data,
        }
    }

    pub fn save(self, mut save_path: PathBuf) {
        let file_name = {
            // Use time to differentiate saves with the same name.
            let time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|time| time.as_secs())
                .unwrap_or(0);

            // Don't hash board data as it might be very large.
            let mut hasher = DefaultHasher::new();
            self.save_name.hash(&mut hasher);
            self.save_description.hash(&mut hasher);
            self.board_area.hash(&mut hasher);

            format!("{}-{}", hasher.finish(), time)
        };
        save_path.set_file_name(file_name);

        save_path.set_extension("save");
        println!("{:?}", save_path);
    }

    pub fn load(path: impl Into<Box<Path>>) -> Result<Self, Box<dyn Error>> {
        let file_data = std::fs::read_to_string(path.into())?;
        Ok(serde_json::from_str(&file_data)?)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// An empty dir must return an empty slice.
    fn empty_dir() -> Result<(), Box<dyn Error>> {
        let temp_dir = tempfile::tempdir()?;
        let parse_saves = parse_saves(temp_dir.path())?;
        assert!(parse_saves.is_empty());
        println!("{:?}", temp_dir.path());
        Ok(())
    }
}
