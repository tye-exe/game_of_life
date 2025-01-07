use egui::{Color32, KeyboardShortcut};
use egui_keybind::Shortcut;

use crate::app::SETTINGS_PANEL;

use super::lang::{
    SETTINGS_CELL_ALIVE_COLOUR, SETTINGS_CELL_DEAD_COLOUR, SETTINGS_CELL_HEADER,
    SETTINGS_CELL_SIZE, SETTINGS_CLOSE, SETTINGS_KEYBIND_HEADER,
    SETTINGS_KEYBIND_SETTINGS_MENU_TOGGLE, SETTINGS_KEYBIND_SIMULATION_TOGGLE, SETTINGS_LABEL,
    SETTINGS_RESET,
};

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
#[serde(default)]
pub(crate) struct Settings {
    #[serde(skip)]
    pub(crate) open: bool,

    /// The settings for cell aperance on the board.
    pub(crate) cell: CellSettings,
    /// The settings for keybinds.
    pub(crate) keybind: KeybindSettings,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub(crate) struct CellSettings {
    #[serde(skip)]
    pub(crate) open: bool,

    /// The colour of alive cells.
    pub(crate) alive_colour: Color32,
    /// The colour of dead cells.
    pub(crate) dead_colour: Color32,
    /// The size of each cell.
    pub(crate) size: f32,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub(crate) struct KeybindSettings {
    #[serde(skip)]
    pub(crate) open: bool,

    /// Keybind for toggling the settings menu.
    pub(crate) settings_menu: Shortcut,
    /// Keybind for toggling the simulation.
    pub(crate) toggle_simulation: Shortcut,
}

impl Settings {
    /// The key used for saving the configuration with [`eframe::set_value`] & [`eframe::get_value`]
    pub(crate) const SAVE_KEY: &str = "game_of_life";
}

impl Settings {
    /// Draw the settings menu if it is open.
    pub(crate) fn draw(&mut self, ctx: &egui::Context) -> Option<egui::InnerResponse<()>> {
        egui::SidePanel::left(SETTINGS_PANEL).show_animated(ctx, self.open, |ui| {
            ui.horizontal(|ui| {
                if ui.button(SETTINGS_CLOSE).clicked() {
                    self.open = false;
                }
                ui.separator();
                ui.label(SETTINGS_LABEL);
            });

            ui.separator();

            self.cell.draw(ui);
            self.keybind.draw(ui);
        })
    }
}

impl Default for CellSettings {
    fn default() -> Self {
        Self {
            alive_colour: Color32::WHITE,
            dead_colour: Color32::BLACK,
            size: 15.0,
            open: false,
        }
    }
}

impl CellSettings {
    fn draw(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(SETTINGS_CELL_HEADER).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(SETTINGS_CELL_ALIVE_COLOUR);
                ui.color_edit_button_srgba(&mut self.alive_colour);
                if ui.small_button(SETTINGS_RESET).clicked() {
                    self.alive_colour = CellSettings::default().alive_colour;
                }
            });

            ui.horizontal(|ui| {
                ui.label(SETTINGS_CELL_DEAD_COLOUR);
                ui.color_edit_button_srgba(&mut self.dead_colour);
                if ui.small_button(SETTINGS_RESET).clicked() {
                    self.dead_colour = CellSettings::default().dead_colour;
                }
            });

            ui.horizontal(|ui| {
                ui.label(SETTINGS_CELL_SIZE);
                ui.add(
                    egui::Slider::new(&mut self.size, 10.0..=50.0)
                        // Allow user override
                        .clamping(egui::SliderClamping::Never),
                );
                if ui.button(SETTINGS_RESET).clicked() {
                    self.size = CellSettings::default().size;
                }
            });
        });
    }
}

impl Default for KeybindSettings {
    fn default() -> Self {
        Self {
            settings_menu: Shortcut::new(
                Some(KeyboardShortcut::new(
                    egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                    egui::Key::D,
                )),
                None,
            ),
            toggle_simulation: Shortcut::new(
                Some(KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::P)),
                None,
            ),
            open: false,
        }
    }
}

impl KeybindSettings {
    fn draw(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(SETTINGS_KEYBIND_HEADER).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(SETTINGS_KEYBIND_SETTINGS_MENU_TOGGLE);
                ui.add(egui_keybind::Keybind::new(
                    &mut self.settings_menu,
                    SETTINGS_KEYBIND_SETTINGS_MENU_TOGGLE,
                ));
            });

            ui.horizontal(|ui| {
                ui.label(SETTINGS_KEYBIND_SIMULATION_TOGGLE);
                ui.add(egui_keybind::Keybind::new(
                    &mut self.toggle_simulation,
                    SETTINGS_KEYBIND_SIMULATION_TOGGLE,
                ));
            });
        });
    }
}
