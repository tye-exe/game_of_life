use gol_lib::persistence::preview::BlueprintPreview;

use super::{LoadMenu, Loadable};
use crate::lang;

lang! {
    LOAD_WINDOW, "Blueprints"
}

pub(crate) type BlueprintLoad = LoadMenu<BlueprintPreview>;

impl Loadable for BlueprintPreview {
    fn format_valid(&self, ui: &mut egui::Ui) {
        ui.label("Todo");
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
        "Load Failed"
    }

    fn delete_success() -> &'static str {
        "Delete success"
    }

    fn delete_fail() -> &'static str {
        "Delete Fail"
    }
}
