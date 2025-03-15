use std::{
    io,
    path::{Path, PathBuf},
};

use egui_toast::{Toast, Toasts};
use gol_lib::persistence::{ParseError, load_preview};
use oneshot::TryRecvError;
use serde::de::DeserializeOwned;

use crate::app::toast_options;

pub(crate) mod blueprint_load;
pub(crate) mod blueprint_save;
pub(crate) mod board_load;
pub(crate) mod board_save;

/// A [oneshot::Receiver] that will receive the status of a file deletion.
type DeleteReceiver = oneshot::Receiver<Box<[(io::Result<()>, PathBuf)]>>;

/// Is used to denote a type that can be displayed in a load menu.
pub(crate) trait Loadable: DeserializeOwned + Send + 'static {
    /// The displayed information for a single instance in a grid of multiple instances.
    fn format_valid(&self, ui: &mut egui::Ui);
    /// The filename, with extension, of this instance.
    fn get_filename(&self) -> String;

    /// A unique ID used to store data about this widget implementation within egui.
    fn id() -> egui::Id;
    /// The name of the window being displayed.
    /// This must be unique.
    fn window_name() -> &'static str;
    /// The message displayed when the file could not be loaded.
    fn load_failed() -> &'static str;
    /// The message displayed when the file was deleted successfully.
    fn delete_success() -> &'static str;
    /// The message displayed when the file deletion failed.
    fn delete_fail() -> &'static str;
}

/// Contains an instance of [`Loadable`] with some extra data.
pub(crate) struct Loaded<T: Loadable> {
    /// The parsed preview.
    pub(crate) preview: Result<T, ParseError>,
    /// Whether the preview has been selected by the user.
    pub(crate) selected: bool,
}

impl<T: Loadable> From<Result<T, ParseError>> for Loaded<T> {
    fn from(value: Result<T, ParseError>) -> Self {
        Self {
            preview: value,
            selected: false,
        }
    }
}

/// The different states the load menu can be in.
#[derive(Default)]
enum State<T: Loadable> {
    /// The preview will be requested.
    #[default]
    Request,
    /// Waiting for the preview response.
    Waiting {
        receiver: oneshot::Receiver<Result<Box<[Result<T, ParseError>]>, io::Error>>,
    },
    /// The loaded save previws.
    Loaded {
        previews: Result<Box<[Loaded<T>]>, io::Error>,
        reload: bool,
        load_selected: bool,
        delete_selected: bool,
        delete_result: Option<DeleteReceiver>,
    },
}

/// A generic load menu displaying elements of type `T`.
pub(crate) struct LoadMenu<T: Loadable> {
    /// Whether the load menu should be shown this frame.
    pub(crate) show: bool,

    /// The internal state of the load menu.
    state: State<T>,
}

impl<T: Loadable> Default for LoadMenu<T> {
    fn default() -> Self {
        Self {
            show: Default::default(),
            state: Default::default(),
        }
    }
}

impl<T: Loadable> LoadMenu<T> {
    /// Shows the load menu if it should be drawn.
    pub(crate) fn draw(&mut self, ctx: &egui::Context) {
        egui::Window::new(T::window_name())
            .open(&mut self.show)
            .show(ctx, |ui| match &mut self.state {
                // If the saves have been loaded draw them.
                State::Loaded {
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
                                .on_disabled_hover_text("Only a single file must be selected")
                                .on_hover_text("Load the selected file.")
                                .clicked();

                            *delete_selected = ui
                                .add_enabled(saves_selected > 0, egui::Button::new("Delete"))
                                .on_disabled_hover_text("More than one file must be selected")
                                .on_hover_text("Delete the selected file(s)")
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
                            egui::Grid::new(format!("{} : GRID", T::window_name()))
                                .striped(true)
                                // The column width has to be manually set to max width
                                .max_col_width(ui.available_size_before_wrap().x)
                                .show(ui, |ui| show_grid(ui, saves));
                        });
                    }
                    Err(err) => {
                        ui.label(format!("Unable to parse files: {err}"));
                    }
                },
                // Shows a spinner whilst waiting for previews.
                State::Waiting { .. } | State::Request => {
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
        match &mut self.state {
            // Requests for the saves to be drawn.
            State::Request => {
                let save_location = save_location.to_path_buf();
                let (tx, rx) = oneshot::channel();

                io_thread.execute(move || {
                    let _ = tx
                        .send(load_preview(save_location.as_path()))
                        .inspect_err(|e| eprintln!("Unable to send load data to GUI: {e}"));
                });

                self.state = State::Waiting { receiver: rx };
            }
            // Check for task completion.
            State::Waiting { receiver } => match receiver.try_recv() {
                Ok(response) => {
                    self.state = State::Loaded {
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
                            .text(T::load_failed()),
                    );
                    // Load an empty list to revent endless failed requests.
                    self.state = State::Loaded {
                        previews: Ok(Box::new([])),
                        reload: false,
                        load_selected: false,
                        delete_selected: false,
                        delete_result: None,
                    };
                }
            },
            State::Loaded {
                reload,
                delete_selected,
                previews,
                delete_result,
                ..
            } => {
                if *reload {
                    self.state = State::Request;
                    return;
                }

                if let (true, Ok(previews)) = (delete_selected, previews) {
                    *delete_result = Some(delete_selected_previews(
                        save_location.into(),
                        previews,
                        io_thread,
                    ));
                }

                delete_response::<T>(toats, delete_result);
            }
        }
    }

    /// Returns the save the preview wants to load. Or `None` if there is no preview to load.
    ///
    /// This method will only return a preview on the update that it was requested. Calling this method on subsequent updates will return `None`.
    pub fn preview_to_load(&mut self) -> Option<&T> {
        match &self.state {
            State::Request | State::Waiting { .. } => return None,
            State::Loaded {
                previews,
                load_selected,
                ..
            } => {
                if let Ok(saves) = previews {
                    if *load_selected {
                        let selected: Vec<&Result<T, ParseError>> = saves
                            .iter()
                            .filter_map(|preview| {
                                if preview.selected {
                                    Some(&preview.preview)
                                } else {
                                    None
                                }
                            })
                            .collect();

                        // Only one preview must be selected.
                        if selected.len() != 1 {
                            return None;
                        }

                        // If any errors are present don't try to parse the save file.
                        return selected.first().and_then(|save| match save {
                            Ok(preview) => Some(preview),
                            Err(_) => None,
                        });
                        // .cloned();
                    }
                }
            }
        };

        None
    }
}

/// The grid structure for displaying `T`.
fn show_grid<T: Loadable>(ui: &mut egui::Ui, saves: &mut Box<[Loaded<T>]>) {
    let mut id = T::id();

    for save in saves {
        let response = ui
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
                            preview.format_valid(ui);
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

/// Changes the given ui to display an invalid save file.
fn format_error(ui: &mut egui::Ui, err: &ParseError) {
    ui.heading(egui::RichText::new("Invalid Data").italics());

    let string_path = err
        .file_path()
        .and_then(|path| path.to_str())
        .unwrap_or("Unable to get path");

    ui.label(format!("Path: {string_path}",));
    ui.small(format!("Error: {err}"));
}

/// Attempt to delete the selected files.
///
/// This function runs the file deletion on the background thread, with the results of each deletion alonside the filepath
/// being sent on the channel returned from this function.
pub(crate) fn delete_selected_previews<T: Loadable>(
    save_location: PathBuf,
    previews: &Box<[Loaded<T>]>,
    io_thread: &threadpool::ThreadPool,
) -> DeleteReceiver {
    // Get the filepath to each file that is selected.
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
        // Try to delete each file.
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

/// Creates toasts depending on the status of deleting files.
pub(crate) fn delete_response<T: Loadable>(
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
                                format!("{} {}", T::delete_success(), path.to_string_lossy());
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
                            let text =
                                format!("{} {}\n{}", T::delete_fail(), path.to_string_lossy(), err);

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
