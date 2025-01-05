use std::path::Path;

use crate::logic::{board_data::SaveData, UiPacket};

#[derive(Default)]
pub(crate) struct Save {
    pub(crate) show: bool,

    pub(crate) save_name: String,
    pub(crate) save_description: String,

    pub(crate) save_requested: bool,
}

impl Save {
    pub(crate) fn draw(&mut self, ctx: &egui::Context, to_send: &mut Vec<UiPacket>) {
        egui::Window::new("a").open(&mut self.show).show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("name:");
                ui.text_edit_singleline(&mut self.save_name);
            });

            if ui.button("Save").clicked() && !self.save_requested {
                self.save_requested = true;
                to_send.push(UiPacket::SaveBoard);
            }

            if self.save_requested {
                ui.spinner();
            }
        });
        // .map(|response| response.inner)
        // .flatten()
        // .unwrap_or(false)
    }

    // pub(crate) fn save_requested(&self) -> bool {
    //     self.save_requested
    // }
}

pub(crate) struct Load {
    pub(crate) show: bool,

    saves: Option<Box<[SaveData]>>,
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
    pub(crate) fn load_saves(&mut self, save_root: &Path) {
        todo!()
    }

    pub(crate) fn draw(&mut self, ctx: &egui::Context) {
        egui::Window::new("load")
            .open(&mut self.show)
            .show(ctx, |ui| {
                if ui.button("Load saves").clicked() {
                    // todo!()
                }
            });
    }
}
