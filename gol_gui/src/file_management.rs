use std::path::Path;

use egui::RichText;
use egui_file_dialog::FileDialog;
use egui_toast::{Toast, Toasts};
use gol_lib::persistence::ParseError;
use gol_lib::persistence::save::BoardSaveError;
use gol_lib::{
    communication::UiPacket,
    persistence::{self, preview::SavePreview},
};
use oneshot::TryRecvError;

use crate::app::toast_options;
use crate::{lang, settings::Settings};

lang! {
    WINDOW, "Save Board";
    NAME, "Name:";
    DESCRIPTION, "Description:";
    BUTTON, "Save";
    LOAD_WINDOW, "Load Board";
    TAGS, "Tags:";
    SAVE_SUCCESS, "Successfully saved board.";
    SAVE_ERROR, "Unable to save board:";
    SAVE_UNKNOWN, "Cannot verify save success.";
    LOAD_FAILED, "Cannot retrieve save previews.";
    NO_SAVES, "There is no saved files.";
    ADD_TAG, "Add a new tag to this save.";
    REMOVE_TAG, "Remove this tag from the save."
}

const LOAD_GRID: &str = "Load Grid";

/// The status regarding user save requests.
#[derive(Default, kinded::Kinded)]
pub enum SaveStatus {
    /// No save is pending.
    #[default]
    Idle,
    /// A save has been requested.
    Request,
    /// Wait for response.
    Waiting {
        response_receiver: oneshot::Receiver<Result<Box<Path>, BoardSaveError>>,
    },
}

pub(crate) struct Save {
    pub(crate) show: bool,
    save_name: String,
    save_description: String,
    save_tags: Vec<String>,

    save_status: SaveStatus,
}

impl Default for Save {
    fn default() -> Self {
        Self {
            show: Default::default(),
            save_name: Default::default(),
            save_description: Default::default(),
            save_tags: vec!["".to_owned()],
            save_status: Default::default(),
        }
    }
}

impl Save {
    pub fn get_name(&self) -> &str {
        &self.save_name
    }

    pub fn get_description(&self) -> &str {
        &self.save_description
    }

    pub fn get_tags(&self) -> &Vec<String> {
        &self.save_tags
    }

    /// Changes the internal state from [SaveStatus::Request] to [SaveStatus::Waiting].
    pub fn set_waiting(
        &mut self,
        response_receiver: oneshot::Receiver<Result<Box<Path>, BoardSaveError>>,
    ) {
        if self.save_status.kind() != SaveStatusKind::Request {
            return;
        }

        self.save_status = SaveStatus::Waiting { response_receiver }
    }

    /// Draws the save menu if it should be shown.
    pub(crate) fn draw(&mut self, ctx: &egui::Context, to_send: &mut Vec<UiPacket>) {
        egui::Window::new(WINDOW)
            .open(&mut (self.show))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    // Gray out the button if a save has been requested.
                    let button = ui.add_enabled(
                        self.save_status.kind() != SaveStatusKind::Waiting,
                        egui::Button::new(BUTTON),
                    );
                    // Only allow one save to be requested at a time.
                    if button.clicked() && self.save_status.kind() == SaveStatusKind::Idle {
                        self.save_status = SaveStatus::Request;
                        to_send.push(UiPacket::SaveBoard);
                    }

                    // Show spinner in right corner.
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Show a spinner whilst waiting for save.
                        if self.save_status.kind() == SaveStatusKind::Waiting {
                            ui.spinner();
                        }
                    });
                });

                ui.separator();

                ui.label(NAME);
                ui.text_edit_singleline(&mut self.save_name);

                ui.label(DESCRIPTION);
                ui.text_edit_multiline(&mut self.save_description);

                ui.label(TAGS);
                self.save_tags.retain_mut(|tag| {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(tag);
                        !ui.small_button("-").on_hover_text(REMOVE_TAG).clicked()
                    })
                    .inner
                });

                if ui.small_button("+").on_hover_text(ADD_TAG).clicked() {
                    self.save_tags.push(String::new());
                }
            });
    }

    /// Updates the internal state.
    ///
    /// This should be run every frame.
    pub fn update(&mut self, ctx: &egui::Context, settings: &mut Settings, toasts: &mut Toasts) {
        // If waiting for a save response, check if there has been a response.
        if let SaveStatus::Waiting { response_receiver } = &self.save_status {
            match response_receiver.try_recv() {
                Ok(response) => {
                    match response {
                        Ok(_) => {
                            toasts.add(
                                Toast::new()
                                    .kind(egui_toast::ToastKind::Info)
                                    .options(toast_options())
                                    .text(SAVE_SUCCESS),
                            );
                        }
                        Err(err) => {
                            toasts.add(
                                Toast::new()
                                    .kind(egui_toast::ToastKind::Error)
                                    .options(toast_options())
                                    .text(format!("{SAVE_ERROR} {err}",)),
                            );
                        }
                    }
                    self.save_status = SaveStatus::Idle;
                }
                Err(TryRecvError::Disconnected) => {
                    toasts.add(
                        Toast::new()
                            .kind(egui_toast::ToastKind::Warning)
                            .options(toast_options())
                            .text(SAVE_UNKNOWN),
                    );

                    self.save_status = SaveStatus::Idle;
                }
                Err(TryRecvError::Empty) => (),
            }
        }
    }
}

/// The different states the load menu can be in.
#[derive(kinded::Kinded, Default)]
enum LoadState {
    /// The preview will be requested.
    #[default]
    Request,
    /// Waiting for the preview response.
    Waiting {
        receiver: oneshot::Receiver<Result<Box<[Result<SavePreview, ParseError>]>, std::io::Error>>,
    },
    /// The loaded save previws.
    Loaded {
        previews: Result<Box<[Preview]>, std::io::Error>,
        reload: bool,
        load_selected: bool,
        delete_selected: bool,
    },
}

/// Contains a parsed preview.
struct Preview {
    /// The parsed preview.
    preview: Result<SavePreview, ParseError>,
    /// Whether the preview has been selected by the user.
    selected: bool,
}

impl From<Result<SavePreview, ParseError>> for Preview {
    fn from(value: Result<SavePreview, ParseError>) -> Self {
        Self {
            preview: value,
            selected: false,
        }
    }
}

/// Responsible for loading and displaying [`SavePreview`]s to the user.
pub(crate) struct Load {
    pub(crate) show: bool,

    saves: LoadState,
}

impl Default for Load {
    fn default() -> Self {
        Self {
            show: false,
            saves: LoadState::Request,
        }
    }
}

impl Load {
    /// Draws the load menu if it is being shown.
    pub(crate) fn draw(&mut self, ctx: &egui::Context) {
        egui::Window::new(LOAD_WINDOW)
            .open(&mut self.show)
            .show(ctx, |ui| match &mut self.saves {
                // If the saves have been loaded draw them.
                LoadState::Loaded {
                    previews: saves,
                    reload,
                    load_selected,
                    delete_selected,
                } => match saves {
                    Ok(saves) => {
                        // The number of selected saves.
                        let saves_selected = saves
                            .iter()
                            .fold(0usize, |num, preview| num + preview.selected as usize);

                        ui.horizontal(|ui| {
                            *load_selected = ui
                                .add_enabled(saves_selected == 1, egui::Button::new("Load"))
                                .on_disabled_hover_text("Only a single save must be selected")
                                .on_hover_text("Load the selected save.")
                                .clicked();

                            *delete_selected = ui
                                .add_enabled(saves_selected > 0, egui::Button::new("Delete"))
                                .on_disabled_hover_text("More than one save must be selected")
                                .on_hover_text("Delete the selected save(s)")
                                .clicked();

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    *reload = ui.button("Reload").clicked();
                                },
                            );
                        });

                        ui.separator();

                        ui.horizontal(|ui| {
                            if ui.button("Select All").clicked() {
                                saves.iter_mut().for_each(|save| save.selected = true);
                            };
                            if ui.button("Deselect All").clicked() {
                                saves.iter_mut().for_each(|save| save.selected = false);
                            };
                        });

                        // Add a defined border between the interaction buttons
                        // and the displayed previews.
                        //
                        // Similar to egui::Separator
                        egui::Frame::none()
                            .fill(ui.visuals().widgets.noninteractive.bg_stroke.color)
                            .rounding(egui::Rounding::same(1.0))
                            .show(ui, |ui| {
                                let available_space = if ui.is_sizing_pass() {
                                    egui::Vec2::ZERO
                                } else {
                                    ui.available_size_before_wrap()
                                };

                                let space = egui::vec2(available_space.x, 10.0);
                                ui.allocate_at_least(space, egui::Sense::hover());
                            });

                        // Draws the parsed saves.
                        egui::ScrollArea::both().show(ui, |ui| {
                            egui::Grid::new(LOAD_GRID)
                                .striped(true)
                                // The column width has to be manually set to max width
                                .max_col_width(ui.available_size_before_wrap().x)
                                .show(ui, show_grid(saves));
                        });
                    }
                    Err(err) => {
                        ui.label(format!("Unable to parse save files: {err}"));
                    }
                },
                // Shows a spinner whilst waiting for previews.
                LoadState::Waiting { .. } | LoadState::Request => {
                    ui.spinner();
                }
            });
    }

    /// Updates the load menu.
    ///
    /// This should be called every frame.
    pub(crate) fn update(
        &mut self,
        io_thread: &threadpool::ThreadPool,
        save_location: &Path,
        toats: &mut Toasts,
    ) {
        match &self.saves {
            // Requests for the saves to be drawn.
            LoadState::Request => {
                let save_location = save_location.to_path_buf();
                let (tx, rx) = oneshot::channel();

                io_thread.execute(move || {
                    let _ = tx
                        .send(persistence::load_preview(save_location.as_path()))
                        .inspect_err(|e| eprintln!("Unable to send load data to GUI: {e}"));
                });

                self.saves = LoadState::Waiting { receiver: rx };
            }
            // Check for task completion.
            LoadState::Waiting { receiver } => match receiver.try_recv() {
                Ok(response) => {
                    self.saves = LoadState::Loaded {
                        previews: response.map(|previews| {
                            let mut vec = Vec::new();
                            for save in previews {
                                vec.push(save.into());
                            }
                            vec.into()
                        }),
                        reload: false,
                        load_selected: false,
                        delete_selected: false,
                    };
                }
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    toats.add(
                        Toast::new()
                            .kind(egui_toast::ToastKind::Error)
                            .options(toast_options())
                            .text(LOAD_FAILED),
                    );
                    // Load an empty list to revent endless failed requests.
                    self.saves = LoadState::Loaded {
                        previews: Ok(Box::new([])),
                        reload: false,
                        load_selected: false,
                        delete_selected: false,
                    };
                }
            },
            LoadState::Loaded { reload, .. } => {
                if *reload {
                    self.saves = LoadState::Request;
                }
            }
        }
    }

    /// Returns the save the user wants to load. Or `None` if there is no save to load.
    ///
    /// This method will only return a save on the update that it was requested. Calling this method on subsequent updates will return `None`.
    pub fn save_to_load(&mut self) -> Option<SavePreview> {
        match &self.saves {
            LoadState::Request | LoadState::Waiting { .. } => return None,
            LoadState::Loaded {
                previews,
                load_selected,
                ..
            } => {
                if let Ok(saves) = previews {
                    if *load_selected {
                        let selected: Vec<&Result<SavePreview, ParseError>> = saves
                            .iter()
                            .filter_map(|preview| {
                                if preview.selected {
                                    Some(&preview.preview)
                                } else {
                                    None
                                }
                            })
                            .collect();

                        // Only one save must be selected.
                        if selected.len() != 1 {
                            return None;
                        }

                        // If any errors are present don't try to parse the save file.
                        return selected
                            .first()
                            .and_then(|save| match save {
                                Ok(preview) => Some(preview),
                                Err(_) => None,
                            })
                            .cloned();
                    }
                }
            }
        };

        None
    }
}

/// Shows the grid of loaded files.
fn show_grid(saves: &mut Box<[Preview]>) -> impl FnOnce(&mut egui::Ui) + use<'_> {
    move |ui| {
        let mut id = egui::Id::new(897234);

        for save in saves {
            let response = &ui
                .vertical(|ui| {
                    // Highlight background if selected
                    let fill = if save.selected {
                        ui.ctx().theme().default_visuals().selection.bg_fill
                    } else {
                        egui::Color32::TRANSPARENT
                    };

                    egui::Frame::default().fill(fill).show(ui, |ui| {
                        match &save.preview {
                            Ok(preview) => {
                                format_valid(ui, preview);
                            }
                            Err(err) => {
                                format_error(ui, err);
                            }
                        }

                        // Expand frame to fill entire area.
                        // This allows the entire area to be highlighted.
                        let available_space = if ui.is_sizing_pass() {
                            egui::Vec2::ZERO
                        } else {
                            ui.available_size_before_wrap()
                        };
                        ui.allocate_at_least(available_space, egui::Sense::hover());
                    });

                    ui.separator();
                })
                .response;

            let interact = ui.interact(response.interact_rect, id, egui::Sense::click());
            // Increment ID to avoid overlap
            id = id.with(90437);

            if interact.clicked() {
                save.selected = !save.selected;
            }

            ui.end_row();
        }
    }
}

/// Changes the given ui to display a valid save file.
fn format_valid(ui: &mut egui::Ui, save: &SavePreview) {
    let text = save.get_name().trim().to_owned();
    if text.is_empty() {
        ui.heading(RichText::new("No Name").italics());
    } else {
        ui.heading(text);
    }

    ui.label(save.get_description());
    ui.label(format!("Generation: {}", save.get_generation()));

    // Get string representation of tags.
    let tags = save
        .get_tags()
        .iter()
        .fold("Tags:".to_owned(), |mut acc, tag| {
            acc.push_str("\n  - ");
            acc.push_str(tag);
            acc
        });
    ui.label(tags);
}

/// Changes the given ui to display an invalid save file.
fn format_error(ui: &mut egui::Ui, err: &ParseError) {
    ui.heading(RichText::new("Invalid Save").italics());

    let string_path = err
        .file_path()
        .and_then(|path| path.to_str())
        .unwrap_or("Unable to get path");

    ui.label(format!("Save Path: {string_path}",));
    ui.small(format!("Error: {err}"));
}
