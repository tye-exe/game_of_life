use crate::app::toast_options;
use crate::lang;
use crate::settings::Settings;
use egui_toast::Toast;
use egui_toast::Toasts;
use gol_lib::communication::UiPacket;
use gol_lib::persistence::save::BoardSaveError;
use oneshot::TryRecvError;
use std::path::Path;
use std::result::Result;

lang! {
    WINDOW, "Save Board";
    NAME, "Name:";
    DESCRIPTION, "Description:";
    BUTTON, "Save";
    ADD_TAG, "Add a new tag to this save.";
    REMOVE_TAG, "Remove this tag from the save.";
    TAGS, "Tags:";
    SAVE_SUCCESS, "Successfully saved board.";
    SAVE_ERROR, "Unable to save board:";
    SAVE_UNKNOWN, "Cannot verify save success."
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

pub(crate) struct Save {
    pub(crate) show: bool,
    pub(crate) save_name: String,
    pub(crate) save_description: String,
    pub(crate) save_tags: Vec<String>,

    pub(crate) save_status: SaveStatus,
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
