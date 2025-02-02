use std::path::Path;

use egui::RichText;
use egui_file_dialog::FileDialog;
use gol_lib::persistence::board_save::BoardSaveError;
use gol_lib::persistence::preview::PreviewParseError;
use gol_lib::{
    communication::UiPacket,
    persistence::{self, preview::SavePreview},
};
use oneshot::TryRecvError;

use crate::{lang, settings::Settings};

lang! {
    WINDOW, "Save Board";
    NAME, "Name:";
    DESCRIPTION, "Description:";
    BUTTON, "Save";
    LOAD_WINDOW, "Load Board"
}

const LOAD_GRID: &str = "Load Grid";

#[derive(thiserror::Error, Debug)]
pub enum ResponseError<Err: std::error::Error> {
    /// The channel disconnected.
    #[error("Receiving on a closed channel.")]
    Disconnected,

    /// The error from the channel.
    #[error(transparent)]
    ResponseError(#[from] Err),
}

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

#[derive(Default)]
pub(crate) struct Save {
    pub(crate) show: bool,
    save_name: String,
    save_description: String,

    save_status: SaveStatus,

    file_dialog: FileDialog,
}

impl Save {
    pub fn get_name(&self) -> &str {
        &self.save_name
    }

    pub fn get_description(&self) -> &str {
        &self.save_description
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
                if ui.button(BUTTON).clicked() && self.save_status.kind() == SaveStatusKind::Idle {
                    self.save_status = SaveStatus::Request;
                    to_send.push(UiPacket::SaveBoard);
                }

                // Show a spinner whilst waiting for save
                if self.save_status.kind() == SaveStatusKind::Waiting {
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

    /// Updates the internal state.
    ///
    /// This should be run every frame.
    pub fn update(&mut self) {
        let outcome;

        // If waiting for a save response, check if their has been a response.
        if let SaveStatus::Waiting { response_receiver } = &self.save_status {
            match response_receiver.try_recv() {
                Ok(response) => outcome = response.map_err(|e| e.into()),
                Err(TryRecvError::Empty) => return,
                Err(TryRecvError::Disconnected) => outcome = Err(ResponseError::Disconnected),
            }
        } else {
            return;
        }

        match outcome {
            Ok(path) => {
                println!("Saved file at {path:?}")
            }
            Err(err) => {
                eprintln!("Unable to save file: {err}")
            }
        }

        self.save_status = SaveStatus::Idle;
    }
}

#[derive(kinded::Kinded, Default)]
enum LoadState {
    /// The preview will be requested.
    #[default]
    Request,
    /// Waiting for the preview response.
    Waiting {
        receiver: oneshot::Receiver<Box<[Result<SavePreview, PreviewParseError>]>>,
    },
    /// The loaded save previws.
    Loaded(Box<[Result<SavePreview, PreviewParseError>]>),
}

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
            .show(ctx, |ui| match &self.saves {
                // If the saves have been loaded draw them.
                LoadState::Loaded(saves) => {
                    egui::ScrollArea::both().show(ui, |ui| {
                        egui::Grid::new(LOAD_GRID)
                            .striped(true)
                            .max_col_width(500.0)
                            .show(ui, show_grid(saves));
                    });
                }
                // Shows a spinner whilst waiting for previews.
                LoadState::Waiting { .. } | LoadState::Request => {
                    ui.spinner();
                }
            });
    }

    /// Updates the load menu.
    ///
    /// This should be called every frame.
    pub(crate) fn update(&mut self, io_thread: &threadpool::ThreadPool, save_location: &Path) {
        match &self.saves {
            // If the saves have been loaded already then no work needs to be done.
            LoadState::Loaded(..) => {}
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
                Ok(response) => self.saves = LoadState::Loaded(response),
                Err(TryRecvError::Empty) => (),
                Err(TryRecvError::Disconnected) => {
                    eprintln!("Io Thread died");
                    self.saves = LoadState::Request
                }
            },
        }
    }
}

/// Shows the grid of loaded files.
fn show_grid(
    saves: &Box<[Result<SavePreview, PreviewParseError>]>,
) -> impl FnOnce(&mut egui::Ui) + use<'_> {
    move |ui| {
        for save in saves {
            ui.vertical(|ui| {
                match save {
                    Ok(save) => {
                        format_valid(ui, save);
                    }
                    Err(err) => {
                        format_error(ui, err);
                    }
                }

                ui.separator();
            });
            ui.end_row();
        }
    }
}

/// Changes the given ui to display a valid save file.
fn format_valid(ui: &mut egui::Ui, save: &SavePreview) {
    let text = save.get_save_name().trim().to_owned();
    if text.is_empty() {
        ui.heading(RichText::new("No Name").italics());
    } else {
        ui.heading(text);
    }

    ui.label(save.get_save_description());
    ui.label(format!("Generation: {}", save.get_generation()));
}

/// Changes the given ui to display an invalid save file.
fn format_error(ui: &mut egui::Ui, err: &PreviewParseError) {
    ui.heading(RichText::new("Invalid Save").italics());

    let string_path = err
        .path()
        .and_then(|path| path.to_str())
        .unwrap_or("Unable to get path");

    ui.label(format!("Save Path: {string_path}",));
    ui.small(format!("Error: {err}"));
}
