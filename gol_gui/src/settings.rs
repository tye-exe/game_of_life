use std::marker::PhantomData;

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
    pub(crate) cell: CellSettings,
    pub(crate) keybind: KeybindSettings,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub(crate) struct CellSettings {
    /// The colour of alive cells.
    pub(crate) alive_colour: Color32,
    /// The colour of dead cells.
    pub(crate) dead_colour: Color32,
    /// The size of each cell.
    pub(crate) size: f32,
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

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub(crate) struct KeybindSettings {
    pub(crate) settings_menu: Shortcut,
    pub(crate) toggle_simulation: Shortcut,
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
        }
    }
}

impl Settings {
    /// The key used for saving the configuration with [`eframe::set_value`] & [`eframe::get_value`]
    pub(crate) const SAVE_KEY: &str = "game_of_life";
}

pub(crate) struct SettingsMenu {
    // Whether the menu is open.
    pub(crate) open: bool,

    // The open sub-menus.
    sub_menus: Box<[Box<dyn Menu>]>,
}

impl SettingsMenu {
    pub(crate) fn draw(
        &mut self,
        settings: &mut Settings,
        ctx: &egui::Context,
    ) -> Option<egui::InnerResponse<()>> {
        egui::SidePanel::left(SETTINGS_PANEL).show_animated(ctx, self.open, |ui| {
            ui.horizontal(|ui| {
                if ui.button(SETTINGS_CLOSE).clicked() {
                    self.open = false;
                }
                ui.separator();
                ui.label(SETTINGS_LABEL);
            });

            ui.separator();

            for menu in &mut self.sub_menus {
                menu.draw(settings, ui);
            }
        })
    }
}

impl Default for SettingsMenu {
    fn default() -> Self {
        Self {
            open: false,
            sub_menus: Box::new([
                Box::new(SubMenu::<Cell>::default()),
                Box::new(SubMenu::<Keybinds>::default()),
            ]),
        }
    }
}

pub(crate) trait Menu {
    fn draw(&mut self, settings: &mut Settings, ui: &mut egui::Ui);
}

struct SubMenu<MenuType> {
    // The open sub-menus
    sub_menus: Box<[Box<dyn Menu>]>,

    _variant: PhantomData<MenuType>,
}

struct Cell {}
struct Keybinds {}

impl Menu for SubMenu<Cell> {
    fn draw(&mut self, settings: &mut Settings, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(SETTINGS_CELL_HEADER).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(SETTINGS_CELL_ALIVE_COLOUR);
                ui.color_edit_button_srgba(&mut settings.cell.alive_colour);
                if ui.small_button(SETTINGS_RESET).clicked() {
                    settings.cell.alive_colour = CellSettings::default().alive_colour;
                }
            });

            ui.horizontal(|ui| {
                ui.label(SETTINGS_CELL_DEAD_COLOUR);
                ui.color_edit_button_srgba(&mut settings.cell.dead_colour);
                if ui.small_button(SETTINGS_RESET).clicked() {
                    settings.cell.dead_colour = CellSettings::default().dead_colour;
                }
            });

            ui.horizontal(|ui| {
                ui.label(SETTINGS_CELL_SIZE);
                ui.add(
                    egui::Slider::new(&mut settings.cell.size, 10.0..=50.0)
                        // Allow user override
                        .clamping(egui::SliderClamping::Never),
                );
                if ui.button(SETTINGS_RESET).clicked() {
                    settings.cell.size = CellSettings::default().size;
                }
            });
        });
    }
}

impl Menu for SubMenu<Keybinds> {
    fn draw(&mut self, settings: &mut Settings, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(SETTINGS_KEYBIND_HEADER).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(SETTINGS_KEYBIND_SETTINGS_MENU_TOGGLE);
                ui.add(egui_keybind::Keybind::new(
                    &mut settings.keybind.settings_menu,
                    SETTINGS_KEYBIND_SETTINGS_MENU_TOGGLE,
                ));
            });

            ui.horizontal(|ui| {
                ui.label(SETTINGS_KEYBIND_SIMULATION_TOGGLE);
                ui.add(egui_keybind::Keybind::new(
                    &mut settings.keybind.toggle_simulation,
                    SETTINGS_KEYBIND_SIMULATION_TOGGLE,
                ));
            });
        });
    }
}

impl Default for SubMenu<Cell> {
    fn default() -> Self {
        Self {
            sub_menus: Box::new([]),
            _variant: PhantomData,
        }
    }
}

impl Default for SubMenu<Keybinds> {
    fn default() -> Self {
        Self {
            sub_menus: Box::new([]),
            _variant: PhantomData,
        }
    }
}
