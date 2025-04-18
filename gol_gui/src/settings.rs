pub mod keybinds;

use std::path::{Path, PathBuf};

use egui::{Color32, InputState};
use egui_file_dialog::FileDialog;
use keybinds::{Keybind, KeybindHolder};
use unicode_segmentation::UnicodeSegmentation;

use crate::{DEFAULT_BLUEPRINT_PATH, DEFAULT_SAVE_PATH, app::SETTINGS_PANEL, lang};

lang! {
        CLOSE, "Close";
        RESET, "Reset";
        LABEL, "Settings";
        CELL_HEADER, "Cells";
        KEYBIND_HEADER, "Keybinds";
        CELL_ALIVE_COLOUR, "Cell Alive Colour:";
        CELL_DEAD_COLOUR, "Cell Dead Colour:";
        CELL_GRID_COLOUR, "Cell Grid Colour:";
        CELL_SELECTION_COLOUR, "Cell Selection Colour:";
        CELL_SIZE, "Cell Size:";
        KEYBIND_SETTINGS_MENU_TOGGLE, "Toggle Settings Menu:";
        KEYBIND_START_SIMULATION, "Start Simulation:";
        KEYBIND_STOP_SIMULATION, "Stop Simulation:";
        KEYBIND_LOAD_BOARD, "Load Board Menu:";
        KEYBIND_LOAD_BLUEPRINT, "Load Blueprint Menu:";
        KEYBIND_SAVE_BOARD, "Save Board Menu:";
        KEYBIND_SAVE_BLUEPRINT, "Save Blueprint Menu:";
        FILE_HEADER, "Storage Locations";
        FILE_SAVE_PATH, "Save Path:";
        FILE_BLUEPRINT_PATH, "Blueprint Path:";
        INTERFACE_HEADER, "Interface";
        THEME_TOGGLE, "Toggle Theme: ";
        TEXT_COLOUR, "Text Colour:";
        WINDOW_COLOUR, "Window Colour:";
        SELECTION_COLOUR, "Text Selection Colour:";
        PANEL_COLOUR, "Panel Colour:";
        NON_INTERACTIVE_BG, "Non Interactive Primary:";
        INACTIVE_BG, "Inactive Primary:";
        OPEN_BG, "Open Primary:";
        NON_INTERACTIVE_WEAK, "None Interactive Secondary:";
        INACTIVE_WEAK, "Inactgive Secondary:";
        OPEN_WEAK, "Open Secondary:";
        HOVERED_BG, "Hovered Primary:";
        ACTIVE_BG, "Active: Primary:";
        HOVERED_WEAK, "Hovered Secondary:";
        ACTIVE_WEAK, "Active Secondary:";
        ZOOM, "Zoom:"
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
    /// The settings for file storage.
    pub(crate) file: FileSettings,
    /// The interface settings.
    pub(crate) interface: InterfaceSettings,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub(crate) struct CellSettings {
    /// The colour of alive cells.
    pub(crate) alive_colour: Color32,
    /// The colour of dead cells.
    pub(crate) dead_colour: Color32,
    /// The colour of the grid lines separating the cells.
    pub(crate) grid_colour: Color32,
    /// The colour of the selection on the cell grid.
    pub(crate) selection_colour: Color32,
    /// The size of each cell.
    pub(crate) size: f32,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub(crate) struct KeybindSettings {
    /// The keybinds and the inputs required to trigger them.
    keybinds: KeybindHolder,

    /// The length of the longest label within this sub-menu.
    /// This is used to align the keybind edit buttons.
    #[serde(skip)]
    longest_label: f32,
    /// The length of the longest shortcut within this sub-menu.
    /// This is used to align the keybind reset buttons.
    #[serde(skip)]
    longest_shortcut: f32,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub(crate) struct FileSettings {
    /// The location of the board saves.
    pub(crate) save_location: PathBuf,
    /// The location of the blueprint saves.
    pub(crate) blueprint_location: PathBuf,

    #[serde(skip)]
    /// .0 : The directory picker for the file locations.
    /// .1 : Whether the selected directory is for saves or blueprints.
    dir_picker: Option<(FileDialog, Selected)>,
}

#[derive(Debug)]
enum Selected {
    Save,
    Blueprint,
}

/// Contains the settings to allow a user to customise the application appearance.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(default)]
pub(crate) struct InterfaceSettings {
    /// The theme overrides for light mode.
    light: StyleOverride,
    /// The theme overrides for dark mode.
    dark: StyleOverride,

    /// The length of the longest label within this sub-menu.
    /// This is used to align the colour edit buttons.
    #[serde(skip)]
    longest_label: f32,
}

/// The overrides for a theme.
#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct StyleOverride {
    text_colour: Color32,
    window_fill: Color32,
    selection_bg: Color32,
    panel_fill: Color32,

    non_interactive_bg: Color32,
    inactive_bg: Color32,
    open_bg: Color32,
    non_interactive_weak: Color32,
    inactive_weak: Color32,
    open_weak: Color32,

    hovered_bg: Color32,
    active_bg: Color32,
    hovered_weak: Color32,
    active_weak: Color32,
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
            self.interface.draw(ui);
        })
    }
}

impl Default for CellSettings {
    fn default() -> Self {
        Self {
            alive_colour: Color32::WHITE,
            dead_colour: Color32::BLACK,
            // Dark blue/purple, more pleasing on the eyes.
            grid_colour: Color32::from_rgb(47, 43, 77),
            size: 15.0,
            selection_colour: Color32::from_rgb(90, 170, 255),
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
                ui.label(CELL_GRID_COLOUR);
                ui.color_edit_button_srgba(&mut self.grid_colour);
                if ui.small_button(RESET).clicked() {
                    self.grid_colour = CellSettings::default().grid_colour;
                }
            });

            ui.horizontal(|ui| {
                ui.label(CELL_SELECTION_COLOUR);
                ui.color_edit_button_srgba(&mut self.selection_colour);
                if ui.small_button(RESET).clicked() {
                    self.selection_colour = CellSettings::default().selection_colour;
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
            keybinds: Default::default(),
            longest_label: 0.0,
            longest_shortcut: 0.0,
        }
    }
}

impl KeybindSettings {
    /// Draws the keybind settings sub-menu.
    fn draw(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(KEYBIND_HEADER).show(ui, |ui| {
            // To change the draw order, change the enum order of "Keybind"
            for keybind in enum_iterator::all::<Keybind>() {
                let label = match keybind {
                    Keybind::SettingsMenu => KEYBIND_SETTINGS_MENU_TOGGLE,
                    Keybind::StartSimulation => KEYBIND_START_SIMULATION,
                    Keybind::StopSimulation => KEYBIND_STOP_SIMULATION,
                    Keybind::LoadBoard => KEYBIND_LOAD_BOARD,
                    Keybind::LoadBlueprint => KEYBIND_LOAD_BLUEPRINT,
                    Keybind::SaveBoard => KEYBIND_SAVE_BOARD,
                    Keybind::SaveBlueprint => KEYBIND_SAVE_BLUEPRINT,
                };

                ui.horizontal(|ui| {
                    let label_width = ui.label(label).rect.max.x;
                    self.longest_label = self.longest_label.max(label_width);
                    // Adds padding so that the end of each label lines up
                    ui.allocate_space(egui::vec2(self.longest_label - label_width, 1.0));

                    let shortcut_width = ui
                        .add(egui_keybind::Keybind::new(
                            self.keybinds.get_shortcut_mut(keybind),
                            label,
                        ))
                        .rect
                        .max
                        .x;
                    self.longest_shortcut = self.longest_shortcut.max(shortcut_width);
                    // Adds padding so that the end of each shortcut lines up
                    ui.allocate_space(egui::vec2(self.longest_shortcut - shortcut_width, 1.0));

                    if ui.small_button(RESET).clicked() {
                        self.keybinds.reset(keybind)
                    }
                });
            }
        });
    }

    /// Returns an iterator of [`Keybind`]s that have been triggered by the given [`InputState`].
    ///
    /// If a keybind has been pressed, then it consumes the input keys. This prevents any other matches.
    /// Due to this keybinds are checked via the most complex keybinds (ones with the most keys) first.
    pub(crate) fn pressed(
        &mut self,
        input_state: &mut InputState,
    ) -> impl Iterator<Item = Keybind> {
        self.keybinds.pressed(input_state)
    }
}

impl Default for FileSettings {
    fn default() -> Self {
        Self {
            save_location: DEFAULT_SAVE_PATH.clone(),
            blueprint_location: DEFAULT_BLUEPRINT_PATH.clone(),
            dir_picker: None,
        }
    }
}

impl FileSettings {
    fn draw(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        egui::CollapsingHeader::new(FILE_HEADER).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(FILE_SAVE_PATH);
                if ui.button(get_display_path(&self.save_location)).clicked() {
                    self.dir_picker = Some((
                        {
                            let mut file_dialog = FileDialog::new();
                            file_dialog.pick_directory();
                            file_dialog
                        },
                        Selected::Save,
                    ));
                }
                if ui.button(RESET).clicked() {
                    self.save_location = DEFAULT_SAVE_PATH.clone();
                }
            });

            ui.horizontal(|ui| {
                ui.label(FILE_BLUEPRINT_PATH);
                if ui
                    .button(get_display_path(&self.blueprint_location))
                    .clicked()
                {
                    self.dir_picker = Some((
                        {
                            let mut file_dialog = FileDialog::new();
                            file_dialog.pick_directory();
                            file_dialog
                        },
                        Selected::Blueprint,
                    ));
                }
                if ui.button(RESET).clicked() {
                    self.blueprint_location = DEFAULT_BLUEPRINT_PATH.clone();
                }
            });

            if let Some((ref mut file_dialog, ref mut selected)) = self.dir_picker {
                file_dialog.update(ctx);

                if let Some(directory) = file_dialog.take_picked() {
                    match selected {
                        Selected::Save => self.save_location = directory.to_path_buf(),
                        Selected::Blueprint => self.blueprint_location = directory.to_path_buf(),
                    }

                    // Dir has been picked so remove dir picker
                    self.dir_picker = None
                };
            }
        });
    }
}

/// If a path is short than 40 characters the full path is returned as a string.
/// Otherwise, the last 40 characters of the path are returned prefixed with "...".
fn get_display_path(path: &Path) -> String {
    let display = path.display().to_string();
    let graphemes: Vec<&str> = display.graphemes(true).collect();

    if 40 >= graphemes.len() {
        return display;
    }

    // Get last 40 chars
    let displayed_path: String = graphemes.into_iter().rev().take(40).rev().collect();
    format!("...{displayed_path}")
}

impl Default for InterfaceSettings {
    fn default() -> Self {
        let dark = egui::Theme::Dark.default_visuals();
        let light = egui::Theme::Light.default_visuals();

        Self {
            light: StyleOverride::from_visual(light),
            dark: StyleOverride::from_visual(dark),
            longest_label: 0.0,
        }
    }
}

impl StyleOverride {
    /// Generates an override with the given visual colours.
    pub(crate) fn from_visual(visual: egui::Visuals) -> Self {
        Self {
            text_colour: visual.text_color(),
            window_fill: visual.window_fill,
            selection_bg: visual.selection.bg_fill,
            panel_fill: visual.panel_fill,
            non_interactive_bg: visual.widgets.noninteractive.bg_fill,
            inactive_bg: visual.widgets.inactive.bg_fill,
            open_bg: visual.widgets.open.bg_fill,
            non_interactive_weak: visual.widgets.noninteractive.weak_bg_fill,
            inactive_weak: visual.widgets.inactive.weak_bg_fill,
            open_weak: visual.widgets.open.weak_bg_fill,
            hovered_bg: visual.widgets.hovered.bg_fill,
            active_bg: visual.widgets.active.bg_fill,
            hovered_weak: visual.widgets.hovered.weak_bg_fill,
            active_weak: visual.widgets.active.weak_bg_fill,
        }
    }
}

impl InterfaceSettings {
    /// Gets the style overrides for the given theme.
    pub(crate) fn get_style(&self, current_theme: egui::Theme) -> &StyleOverride {
        match current_theme {
            egui::Theme::Dark => &self.dark,
            egui::Theme::Light => &self.light,
        }
    }

    /// Gets mutable style overrides for the given theme.
    pub(crate) fn get_style_mut(&mut self, current_theme: egui::Theme) -> &mut StyleOverride {
        match current_theme {
            egui::Theme::Dark => &mut self.dark,
            egui::Theme::Light => &mut self.light,
        }
    }

    /// Applies the current style overrides to the entire gui.
    pub(crate) fn apply_style(&self, ctx: &egui::Context) {
        let style_override = self.get_style(ctx.theme());

        ctx.style_mut(|style| {
            style.visuals.override_text_color = Some(style_override.text_colour);
            style.visuals.window_fill = style_override.window_fill;
            style.visuals.selection.bg_fill = style_override.selection_bg;
            style.visuals.panel_fill = style_override.panel_fill;

            style.visuals.widgets.noninteractive.bg_fill = style_override.non_interactive_bg;
            style.visuals.widgets.inactive.bg_fill = style_override.inactive_bg;
            style.visuals.widgets.open.bg_fill = style_override.open_bg;
            style.visuals.widgets.noninteractive.weak_bg_fill = style_override.non_interactive_weak;
            style.visuals.widgets.inactive.weak_bg_fill = style_override.inactive_weak;
            style.visuals.widgets.open.weak_bg_fill = style_override.open_weak;

            style.visuals.widgets.hovered.bg_fill = style_override.hovered_bg;
            style.visuals.widgets.active.bg_fill = style_override.active_bg;
            style.visuals.widgets.hovered.weak_bg_fill = style_override.hovered_weak;
            style.visuals.widgets.active.weak_bg_fill = style_override.active_weak;
        });
    }

    /// Draws the interface settings sub-menu.
    pub(crate) fn draw(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new(INTERFACE_HEADER).show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(ZOOM);

                let mut zoom_factor = ui.ctx().zoom_factor();
                if ui.small_button("-").clicked() {
                    zoom_factor -= 0.05;
                }
                ui.label(format!("{zoom_factor:.2}"));
                if ui.small_button("+").clicked() {
                    zoom_factor += 0.05;
                }
                // Ensure the zoom factor cannot go too low.
                // Otherwise a panic will occur.
                zoom_factor = zoom_factor.max(0.1);

                if ui.small_button(RESET).clicked() {
                    zoom_factor = 1.0;
                }

                ui.ctx().set_zoom_factor(zoom_factor);
            });

            ui.separator();

            ui.horizontal(|ui| {
                ui.label(THEME_TOGGLE);
                egui::global_theme_preference_buttons(ui);
            });

            ui.separator();

            let current_theme = ui.ctx().theme();
            let mut longest_label = self.longest_label;
            let style_override = self.get_style_mut(current_theme);

            ui.horizontal(|ui| {
                let label_width = ui.label(TEXT_COLOUR).rect.max.x;
                longest_label = longest_label.max(label_width);
                ui.allocate_space(egui::vec2(longest_label - label_width, 1.0));

                ui.color_edit_button_srgba(&mut style_override.text_colour);
                if ui.small_button(RESET).clicked() {
                    style_override.text_colour = InterfaceSettings::default()
                        .get_style(current_theme)
                        .text_colour;
                }
            });

            ui.horizontal(|ui| {
                let label_width = ui.label(WINDOW_COLOUR).rect.max.x;
                longest_label = longest_label.max(label_width);
                ui.allocate_space(egui::vec2(longest_label - label_width, 1.0));

                ui.color_edit_button_srgba(&mut style_override.window_fill);
                if ui.small_button(RESET).clicked() {
                    style_override.window_fill = InterfaceSettings::default()
                        .get_style(current_theme)
                        .window_fill;
                }
            });

            ui.horizontal(|ui| {
                let label_width = ui.label(SELECTION_COLOUR).rect.max.x;
                longest_label = longest_label.max(label_width);
                ui.allocate_space(egui::vec2(longest_label - label_width, 1.0));

                ui.color_edit_button_srgba(&mut style_override.selection_bg);
                if ui.small_button(RESET).clicked() {
                    style_override.selection_bg = InterfaceSettings::default()
                        .get_style(current_theme)
                        .selection_bg;
                }
            });

            ui.horizontal(|ui| {
                let label_width = ui.label(PANEL_COLOUR).rect.max.x;
                longest_label = longest_label.max(label_width);
                ui.allocate_space(egui::vec2(longest_label - label_width, 1.0));

                ui.color_edit_button_srgba(&mut style_override.panel_fill);
                if ui.small_button(RESET).clicked() {
                    style_override.panel_fill = InterfaceSettings::default()
                        .get_style(current_theme)
                        .panel_fill;
                }
            });

            ui.horizontal(|ui| {
                let label_width = ui.label(NON_INTERACTIVE_BG).rect.max.x;
                longest_label = longest_label.max(label_width);
                ui.allocate_space(egui::vec2(longest_label - label_width, 1.0));

                ui.color_edit_button_srgba(&mut style_override.non_interactive_bg);
                if ui.small_button(RESET).clicked() {
                    style_override.non_interactive_bg = InterfaceSettings::default()
                        .get_style(current_theme)
                        .non_interactive_bg;
                }
            });

            ui.horizontal(|ui| {
                let label_width = ui.label(NON_INTERACTIVE_WEAK).rect.max.x;
                longest_label = longest_label.max(label_width);
                ui.allocate_space(egui::vec2(longest_label - label_width, 1.0));

                ui.color_edit_button_srgba(&mut style_override.non_interactive_weak);
                if ui.small_button(RESET).clicked() {
                    style_override.non_interactive_weak = InterfaceSettings::default()
                        .get_style(current_theme)
                        .non_interactive_bg;
                }
            });

            ui.horizontal(|ui| {
                let label_width = ui.label(INACTIVE_BG).rect.max.x;
                longest_label = longest_label.max(label_width);
                ui.allocate_space(egui::vec2(longest_label - label_width, 1.0));

                ui.color_edit_button_srgba(&mut style_override.inactive_bg);
                if ui.small_button(RESET).clicked() {
                    style_override.inactive_bg = InterfaceSettings::default()
                        .get_style(current_theme)
                        .inactive_bg;
                }
            });

            ui.horizontal(|ui| {
                let label_width = ui.label(INACTIVE_WEAK).rect.max.x;
                longest_label = longest_label.max(label_width);
                ui.allocate_space(egui::vec2(longest_label - label_width, 1.0));

                ui.color_edit_button_srgba(&mut style_override.inactive_weak);
                if ui.small_button(RESET).clicked() {
                    style_override.inactive_weak = InterfaceSettings::default()
                        .get_style(current_theme)
                        .inactive_weak;
                }
            });

            ui.horizontal(|ui| {
                let label_width = ui.label(OPEN_BG).rect.max.x;
                longest_label = longest_label.max(label_width);
                ui.allocate_space(egui::vec2(longest_label - label_width, 1.0));

                ui.color_edit_button_srgba(&mut style_override.open_bg);
                if ui.small_button(RESET).clicked() {
                    style_override.open_bg = InterfaceSettings::default()
                        .get_style(current_theme)
                        .open_bg;
                }
            });

            ui.horizontal(|ui| {
                let label_width = ui.label(OPEN_WEAK).rect.max.x;
                longest_label = longest_label.max(label_width);
                ui.allocate_space(egui::vec2(longest_label - label_width, 1.0));

                ui.color_edit_button_srgba(&mut style_override.open_weak);
                if ui.small_button(RESET).clicked() {
                    style_override.open_weak = InterfaceSettings::default()
                        .get_style(current_theme)
                        .open_weak;
                }
            });

            ui.horizontal(|ui| {
                let label_width = ui.label(HOVERED_BG).rect.max.x;
                longest_label = longest_label.max(label_width);
                ui.allocate_space(egui::vec2(longest_label - label_width, 1.0));

                ui.color_edit_button_srgba(&mut style_override.hovered_bg);
                if ui.small_button(RESET).clicked() {
                    style_override.hovered_bg = InterfaceSettings::default()
                        .get_style(current_theme)
                        .hovered_bg;
                }
            });

            ui.horizontal(|ui| {
                let label_width = ui.label(HOVERED_WEAK).rect.max.x;
                longest_label = longest_label.max(label_width);
                ui.allocate_space(egui::vec2(longest_label - label_width, 1.0));

                ui.color_edit_button_srgba(&mut style_override.hovered_weak);
                if ui.small_button(RESET).clicked() {
                    style_override.hovered_weak = InterfaceSettings::default()
                        .get_style(current_theme)
                        .hovered_weak;
                }
            });

            ui.horizontal(|ui| {
                let label_width = ui.label(ACTIVE_BG).rect.max.x;
                longest_label = longest_label.max(label_width);
                ui.allocate_space(egui::vec2(longest_label - label_width, 1.0));

                ui.color_edit_button_srgba(&mut style_override.active_bg);
                if ui.small_button(RESET).clicked() {
                    style_override.active_bg = InterfaceSettings::default()
                        .get_style(current_theme)
                        .active_bg;
                }
            });

            ui.horizontal(|ui| {
                let label_width = ui.label(ACTIVE_WEAK).rect.max.x;
                longest_label = longest_label.max(label_width);
                ui.allocate_space(egui::vec2(longest_label - label_width, 1.0));

                ui.color_edit_button_srgba(&mut style_override.active_weak);
                if ui.small_button(RESET).clicked() {
                    style_override.active_weak = InterfaceSettings::default()
                        .get_style(current_theme)
                        .active_weak;
                }
            });

            self.longest_label = longest_label;
        });
    }
}
