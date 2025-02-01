use std::path::Path;

use egui_file_dialog::FileDialog;
use gol_lib::persistence::preview::PreviewParseError;
use gol_lib::{
    communication::UiPacket,
    persistence::{self, preview::SavePreview},
};

use crate::{lang, settings::Settings};

lang! {
    WINDOW, "Save Board";
    NAME, "Name:";
    DESCRIPTION, "Description:";
    BUTTON, "Save"
}

#[derive(Default)]
pub(crate) struct Save {
    pub(crate) show: bool,
    pub(crate) save_name: String,
    pub(crate) save_description: String,

    pub(crate) save_requested: bool,

    file_dialog: FileDialog,
}

impl Save {
    pub(crate) fn draw(
        &mut self,
        ctx: &egui::Context,
        to_send: &mut Vec<UiPacket>,
        settings: &mut Settings,
    ) {
        egui::Window::new(WINDOW)
            .open(&mut (self.show))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(NAME);
                    ui.text_edit_singleline(&mut self.save_name);
                });

                ui.horizontal(|ui| {
                    ui.label(DESCRIPTION);
                    ui.text_edit_singleline(&mut self.save_description);
                });

                if ui.button("Folder").clicked() {
                    self.file_dialog = FileDialog::new();
                    self.file_dialog.pick_directory();
                }

                // Only allow one save to be requested at a time
                if ui.button(BUTTON).clicked() && !self.save_requested {
                    self.save_requested = true;
                    to_send.push(UiPacket::SaveBoard);
                }

                // Show a spinner whilst waiting for save
                if self.save_requested {
                    ui.spinner();
                }

                // Constrain the file picker to the save directory
                if let Some(directory) = self.file_dialog.active_entry() {
                    let inside_save = directory
                        .as_path()
                        .canonicalize()
                        .map(|dir_path| {
                            dir_path
                                .to_path_buf()
                                .starts_with(settings.file.save_location.clone())
                        })
                        // Being constrained is not critical so "fail open"
                        .unwrap_or(true);

                    if !inside_save {
                        self.file_dialog = FileDialog::new()
                            .initial_directory(settings.file.save_location.clone());
                        self.file_dialog.pick_directory();
                    }
                }

                self.file_dialog.update(ctx);
            });
    }
}

pub(crate) struct Load {
    pub(crate) show: bool,

    saves: Option<Box<[Result<SavePreview, PreviewParseError>]>>,
}

impl Default for Load {
    fn default() -> Self {
        Self {
            show: false,
            saves: None,
        }
    }
}

impl Load {
    pub(crate) fn draw(&mut self, ctx: &egui::Context, save_location: &Path) {
        egui::Window::new("load")
            .open(&mut self.show)
            .show(ctx, |ui| {
                let mut text = String::new();
                match &mut self.saves {
                    None => self.saves = Some(persistence::load_preview(save_location)),
                    Some(saves) => {
                        for save in saves {
                            match save {
                                Ok(save) => {
                                    text.push_str(&save.save_name);
                                    text.push('\n');
                                    text.push_str(&save.save_description);
                                    text.push('\n');
                                }
                                Err(err) => {
                                    let string_path = err
                                        .path()
                                        .and_then(|path| path.to_str())
                                        .unwrap_or("Unknown file location");

                                    text.push_str(&format!("Error parsing {string_path}\n"));
                                    text.push_str(&format!("Error: {err}"));
                                    text.push('\n');
                                }
                            }
                        }

                        ui.text_edit_multiline(&mut text);
                    }
                }
            });
    }
}
