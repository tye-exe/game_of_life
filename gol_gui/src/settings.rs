use egui::{Color32, KeyboardShortcut};
use egui_keybind::Shortcut;

use crate::{app::SETTINGS_PANEL, lang, USER_BLUEPRINT_PATH, USER_SAVE_PATH};

lang! {
        CLOSE, "Close";
        RESET, "Reset";
        LABEL, "Settings";
        CELL_HEADER, "Cells";
        KEYBIND_HEADER, "Keybinds";
        CELL_ALIVE_COLOUR, "Cell alive colour:";
        CELL_DEAD_COLOUR, "Cell dead colour:";
        CELL_SIZE, "Cell size:";
        KEYBIND_SIMULATION_TOGGLE, "Toggle Simulation:";
        KEYBIND_SETTINGS_MENU_TOGGLE, "Toggle Settings Menu:";
        FILE_HEADER, "Storage locations"
}

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
                if ui.button(CLOSE).clicked() {
                    self.open = false;
                }
                ui.separator();
                ui.label(LABEL);
            });

            ui.separator();

            self.cell.draw(ui);
            self.keybind.draw(ui);
            self.file.draw(ui, ctx);
        })
    }
}

impl Default for CellSettings {
    fn default() -> Self {
        Self {
            alive_colour: Color32::WHITE,
            dead_colour: Color32::BLACK,
            size: 15.0,
        }
    }
}

impl CellSettings {
    fn draw(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(CELL_HEADER).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(CELL_ALIVE_COLOUR);
                ui.color_edit_button_srgba(&mut self.alive_colour);
                if ui.small_button(RESET).clicked() {
                    self.alive_colour = CellSettings::default().alive_colour;
                }
            });

            ui.horizontal(|ui| {
                ui.label(CELL_DEAD_COLOUR);
                ui.color_edit_button_srgba(&mut self.dead_colour);
                if ui.small_button(RESET).clicked() {
                    self.dead_colour = CellSettings::default().dead_colour;
                }
            });

            ui.horizontal(|ui| {
                ui.label(CELL_SIZE);
                ui.add(
                    egui::Slider::new(&mut self.size, 10.0..=50.0)
                        // Allow user override
                        .clamping(egui::SliderClamping::Never),
                );
                if ui.button(RESET).clicked() {
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
        egui::CollapsingHeader::new(KEYBIND_HEADER).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(KEYBIND_SETTINGS_MENU_TOGGLE);
                ui.add(egui_keybind::Keybind::new(
                    &mut self.settings_menu,
                    KEYBIND_SETTINGS_MENU_TOGGLE,
                ));
            });

            ui.horizontal(|ui| {
                ui.label(KEYBIND_SIMULATION_TOGGLE);
                ui.add(egui_keybind::Keybind::new(
                    &mut self.toggle_simulation,
                    KEYBIND_SIMULATION_TOGGLE,
                ));
            });
        });
    }
}
