use crate::app::toast_options;
use crate::lang;
use egui::RichText;
use egui_toast::{Toast, Toasts};
use gol_lib::persistence::ParseError;
use gol_lib::persistence::{self, preview::SavePreview};
use oneshot::TryRecvError;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use std::result::Result;

lang! {
    LOAD_WINDOW, "Load Board";
    LOAD_FAILED, "Cannot retrieve save previews.";
    DELETE_FILE_SUCCESS, "Successfully delete file:";
    DELETE_FILE_ERROR, "Failed to delete file:"
}

pub(crate) const LOAD_GRID: &str = "Load Grid";

/// The different states the load menu can be in.
#[derive(kinded::Kinded, Default)]
pub(crate) enum LoadState {
    /// The preview will be requested.
    #[default]
    Request,
    /// Waiting for the preview response.
    Waiting {
        receiver: oneshot::Receiver<Result<Box<[Result<SavePreview, ParseError>]>, io::Error>>,
    },
    /// The loaded save previws.
    Loaded {
        previews: Result<Box<[Preview]>, io::Error>,
        reload: bool,
        load_selected: bool,
        delete_selected: bool,
        delete_result: Option<DeleteReceiver>,
    },
}

/// Contains a parsed preview.
pub(crate) struct Preview {
    /// The parsed preview.
    pub(crate) preview: Result<SavePreview, ParseError>,
    /// Whether the preview has been selected by the user.
    pub(crate) selected: bool,
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

    pub(crate) saves: LoadState,
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
                    ..
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
                        egui::Frame::NONE
                            .fill(ui.visuals().widgets.noninteractive.bg_stroke.color)
                            .corner_radius(egui::CornerRadius::same(1))
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
        match &mut self.saves {
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
                        delete_result: None,
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
                        delete_result: None,
                    };
                }
            },
            LoadState::Loaded {
                reload,
                delete_selected,
                previews,
                delete_result,
                ..
            } => {
                if *reload {
                    self.saves = LoadState::Request;
                    return;
                }

                if let (true, Ok(previews)) = (delete_selected, previews) {
                    *delete_result = Some(delete_selected_saves(
                        save_location.into(),
                        previews,
                        io_thread,
                    ));
                }

                delete_response(toats, delete_result);
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

/// Creates toasts depending on the status of deleting files.
pub(crate) fn delete_response(
    toats: &mut Toasts,
    delete_result: &mut Option<oneshot::Receiver<Box<[(Result<(), io::Error>, PathBuf)]>>>,
) {
    if let Some(receiver) = delete_result {
        // Check if the file deletion has finished
        match receiver.try_recv() {
            Ok(results) => {
                // Loop over the result for each deletion
                for (status, path) in results {
                    match status {
                        Ok(_) => {
                            let text =
                                format!("{} {}", DELETE_FILE_SUCCESS, path.to_string_lossy());
                            toats.add(
                                Toast::new()
                                    .kind(egui_toast::ToastKind::Success)
                                    .options(toast_options())
                                    .text(text),
                            );
                        }
                        Err(err) => {
                            let options = egui_toast::ToastOptions::default()
                                .duration(None)
                                .show_icon(true);
                            let text = format!(
                                "{} {}\n{}",
                                DELETE_FILE_ERROR,
                                path.to_string_lossy(),
                                err
                            );

                            toats.add(
                                Toast::new()
                                    .kind(egui_toast::ToastKind::Error)
                                    .options(options)
                                    .text(text),
                            );
                        }
                    }
                }

                *delete_result = None;
            }
            Err(oneshot::TryRecvError::Empty) => {}
            Err(oneshot::TryRecvError::Disconnected) => {
                *delete_result = None;
            }
        }
    }
}

pub(crate) type DeleteReceiver = oneshot::Receiver<Box<[(Result<(), std::io::Error>, PathBuf)]>>;

/// Attempt to delete the selected save files.
///
/// This function runs the file deletion on the background thread, with the results of each deletion alonside the filepath
/// being sent on the channel returned from this function.
pub(crate) fn delete_selected_saves(
    save_location: PathBuf,
    previews: &Box<[Preview]>,
    io_thread: &threadpool::ThreadPool,
) -> DeleteReceiver {
    // Get the filepath to each save that is selected.
    // This is done on main thread as the preview data is behind a reference.
    let paths: Vec<PathBuf> = previews
        .iter()
        .filter_map(|preview| {
            if !preview.selected {
                return None;
            }

            // Try to get the path of the save, even if it's invalid.
            match &preview.preview {
                Ok(preview) => {
                    let mut save_folder = save_location.clone();
                    save_folder.push(preview.get_filename());
                    Some(save_folder)
                }
                Err(err) => err.file_path().map(|path| path.to_path_buf()),
            }
        })
        .collect();

    let (sender, receiver) = oneshot::channel();

    // Run file delete on background thread.
    io_thread.execute(|| {
        // Try to delete each save file.
        let results: Box<[(io::Result<()>, PathBuf)]> = paths
            .into_iter()
            .map(|path| (std::fs::remove_file(path.as_path()), path))
            .collect();

        let _ = sender
            .send(results)
            .inspect_err(|_| eprintln!("Unable to send deletion result to GUI"));
    });

    receiver
}

/// Shows the grid of loaded files.
pub(crate) fn show_grid(saves: &mut Box<[Preview]>) -> impl FnOnce(&mut egui::Ui) + use<'_> {
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
pub(crate) fn format_valid(ui: &mut egui::Ui, save: &SavePreview) {
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
pub(crate) fn format_error(ui: &mut egui::Ui, err: &ParseError) {
    ui.heading(RichText::new("Invalid Save").italics());

    let string_path = err
        .file_path()
        .and_then(|path| path.to_str())
        .unwrap_or("Unable to get path");

    ui.label(format!("Save Path: {string_path}",));
    ui.small(format!("Error: {err}"));
}
