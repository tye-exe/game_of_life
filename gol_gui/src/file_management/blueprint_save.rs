use std::path::{Path, PathBuf};

use egui_toast::{Toast, ToastKind, ToastOptions, Toasts};
use gol_lib::{
    Area,
    communication::UiPacket,
    persistence::{SaveBuilder, SimulationBlueprint, save::SaveError},
};
use oneshot::TryRecvError;
use threadpool::ThreadPool;

use crate::{
    app::{edit::Selection, toast_options},
    lang,
};

lang! {
    WINDOW, "Blueprint save";
    SAVE, "Save";
    NO_SELECTION, "No area is selected!";
    NAME, "Name:";
    DESCRIPTION, "Description";
    TAGS, "Tags:";
    SAVE_SUCCESS, "Successfully saved the blueprint.";
    SAVE_ERROR, "Unable to save the blueprint.";
    SAVE_UNKNOWN, "Cannot verify save success.";
    ADD_TAG, "Add a tag to this blueprint.";
    REMOVE_TAG, "Remove a tag from this blueprint."
}

/// The possible states the blueprint save can be in.
pub(crate) enum State {
    /// No save has been requested
    Idle {
        /// The area selected by the user.
        selection: Option<Selection>,
    },
    /// Request that a save be made with the current selection.
    Request {
        /// The selection that the user wants to save.
        selection: Selection,
    },
    /// Waiting to receive the board data from the simulator.
    Waiting,
    /// Waiting to receive the blueprint save response from the IO thread.
    Saving {
        receiver: oneshot::Receiver<Result<Box<Path>, SaveError>>,
    },
}

impl Default for State {
    fn default() -> Self {
        Self::Idle { selection: None }
    }
}

/// Handles the saving of blueprints.
pub(crate) struct BlueprintSave {
    /// Whether or not the blueprint save window is being shown.
    pub(crate) show: bool,

    name: String,
    description: String,
    tags: Vec<String>,

    /// The state the blueprint save is in.
    state: State,
}

impl Default for BlueprintSave {
    fn default() -> Self {
        Self {
            show: Default::default(),
            name: Default::default(),
            description: Default::default(),
            tags: vec!["".to_owned()],
            state: Default::default(),
        }
    }
}

impl BlueprintSave {
    /// Draws the blueprint save window if it should be shown.
    pub(crate) fn draw(&mut self, ctx: &egui::Context) {
        egui::Window::new(WINDOW)
            .open(&mut self.show)
            .show(ctx, |ui| {
                draw_top(&mut self.state, ui);
                ui.separator();

                ui.heading(NAME);
                ui.text_edit_singleline(&mut self.name);

                ui.heading(DESCRIPTION);
                ui.text_edit_multiline(&mut self.description);

                ui.heading(TAGS);
                self.tags.retain_mut(|tag| {
                    ui.horizontal(|ui| {
                        ui.text_edit_singleline(tag);
                        !ui.small_button("-").on_hover_text(REMOVE_TAG).clicked()
                    })
                    .inner
                });

                if ui.small_button("+").on_hover_text(ADD_TAG).clicked() {
                    self.tags.push(String::new());
                }
            });
    }

    /// Updates the internal values for the blueprint save progress.
    /// This should be called every frame.
    pub(crate) fn update(
        &mut self,
        selection: Option<Selection>,
        to_send: &mut Vec<UiPacket>,
        toast: &mut Toasts,
    ) {
        match self.state {
            State::Idle {
                selection: ref mut internal_selection,
            } => *internal_selection = selection,
            State::Request { selection } => {
                to_send.push(UiPacket::SaveBlueprint {
                    area: Area::new(selection.get_start(), selection.get_end()),
                });

                self.state = State::Waiting;
            }
            State::Waiting => {}
            State::Saving { ref receiver } => match receiver.try_recv() {
                Err(TryRecvError::Empty) => {}
                Ok(result) => match result {
                    Ok(_) => {
                        toast.add(
                            Toast::new()
                                .kind(ToastKind::Success)
                                .options(toast_options())
                                .text(SAVE_SUCCESS),
                        );

                        self.state = Default::default();
                    }

                    Err(err) => {
                        toast.add(
                            Toast::new()
                                .kind(ToastKind::Error)
                                .options(ToastOptions::default().duration(None).show_icon(true))
                                .text(format!("{SAVE_ERROR} {err}")),
                        );

                        self.state = Default::default();
                    }
                },
                Err(TryRecvError::Disconnected) => {
                    toast.add(
                        Toast::new()
                            .kind(ToastKind::Error)
                            .options(ToastOptions::default().duration(None).show_icon(true))
                            .text(SAVE_UNKNOWN),
                    );

                    self.state = Default::default();
                }
            },
        }
    }

    /// Saves the given [`SimulationBlueprint`] if the struct is in the correct state for saving a blueprint.
    pub(crate) fn save_blueprint(
        &mut self,
        blueprint: SimulationBlueprint,
        io_thread: &ThreadPool,
        blueprint_path: PathBuf,
    ) {
        if let State::Waiting = self.state {
            let (tx, rx) = oneshot::channel();

            {
                let name = self.name.clone();
                let description = self.description.clone();
                let tags = self.tags.clone().into_boxed_slice();

                // Run task in IO thread
                io_thread.execute(move || {
                    let _ = tx
                        .send(
                            SaveBuilder::new_blueprint(blueprint)
                                .name(name)
                                .desciprtion(description)
                                .tags(tags)
                                .save(blueprint_path),
                        )
                        .inspect_err(|e| eprintln!("Could not communicate with ui thread: {e}"));
                });
            }

            self.state = State::Saving { receiver: rx }
        }
    }
}

/// Draws the top part of the blueprint save window.
fn draw_top(state: &mut State, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        if let State::Idle {
            selection: Some(selection),
        } = state
        {
            if ui.button(SAVE).clicked() {
                *state = State::Request {
                    selection: *selection,
                }
            }

            return;
        }

        // Prevent interaction if a save cannot be started.
        ui.add_enabled(false, egui::Button::new(SAVE));

        ui.with_layout(
            egui::Layout::right_to_left(egui::Align::Center),
            |ui| match state {
                State::Idle { .. } => ui.label(NO_SELECTION),
                _ => ui.spinner(),
            },
        );
    });
}
