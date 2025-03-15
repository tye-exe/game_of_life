use crate::lang;
use egui::RichText;
use gol_lib::persistence::ParseError;
use gol_lib::persistence::preview::SavePreview;
use std::result::Result;

use super::{LoadMenu, Loadable, State};

lang! {
    LOAD_WINDOW, "Load Board";
    LOAD_FAILED, "Cannot retrieve save previews.";
    DELETE_FILE_SUCCESS, "Successfully delete file:";
    DELETE_FILE_ERROR, "Failed to delete file:"
}

pub(crate) type LoadBoard = LoadMenu<SavePreview>;

impl Loadable for SavePreview {
    fn format_valid(&self, ui: &mut egui::Ui) {
        let text = self.get_name().trim().to_owned();
        if text.is_empty() {
            ui.heading(RichText::new("No Name").italics());
        } else {
            ui.heading(text);
        }

        ui.label(self.get_description());
        ui.label(format!("Generation: {}", self.get_generation()));

        // Get string representation of tags.
        let tags = self
            .get_tags()
            .iter()
            .fold("Tags:".to_owned(), |mut acc, tag| {
                acc.push_str("\n  - ");
                acc.push_str(tag);
                acc
            });
        ui.label(tags);
    }

    fn id() -> egui::Id {
        egui::Id::new(534678439)
    }

    fn window_name() -> &'static str {
        LOAD_WINDOW
    }

    fn get_filename(&self) -> String {
        self.get_filename()
    }

    fn load_failed() -> &'static str {
        LOAD_FAILED
    }

    fn delete_success() -> &'static str {
        DELETE_FILE_SUCCESS
    }

    fn delete_fail() -> &'static str {
        DELETE_FILE_ERROR
    }
}

impl LoadMenu<SavePreview> {}
