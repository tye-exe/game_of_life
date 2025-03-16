use egui::RichText;
use gol_lib::persistence::preview::BlueprintPreview;

use super::{LoadMenu, Loadable};
use crate::lang;

lang! {
    LOAD_WINDOW, "Blueprints";
    LOAD_FAILED, "Cannot retrieve blueprint previews.";
    DELETE_SUCCESS, "Successfully deleted blueprint:";
    DELETE_ERROR, "Failed to delete blueprint:"
}

pub(crate) type BlueprintLoad = LoadMenu<BlueprintPreview>;

impl Loadable for BlueprintPreview {
    fn format_valid(&self, ui: &mut egui::Ui) {
        let text = self.get_name().trim().to_owned();
        if text.is_empty() {
            ui.heading(RichText::new("No Name").italics());
        } else {
            ui.heading(text);
        }

        ui.label(self.get_description());

        ui.label(format!(
            "Size: {} x by {} y",
            self.get_x_size(),
            self.get_y_size()
        ));

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
        egui::Id::new(9234)
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
        DELETE_SUCCESS
    }

    fn delete_fail() -> &'static str {
        DELETE_ERROR
    }
}
