#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

pub(crate) mod edit;

use crate::{
    file_management::{
        blueprint_load::BlueprintLoad, blueprint_save::BlueprintSave, board_load::LoadBoard,
        board_save::Save,
    },
    lang,
    settings::{Settings, keybinds::Keybind},
    user_actions::{Action, History},
};
use edit::{EditState, Selection, draw_interaction, preview_interaction, select_interaction};
use egui::{CornerRadius, Id, Painter, Pos2, Rect, epaint::RectShape, pos2};
use egui_keybind::Bind;
use egui_toast::{Toast, Toasts};
use gol_lib::{
    Area, BoardDisplay, Cell, GlobalPosition, SharedDisplay, SimulatorReceiver, UiSender,
    communication::{SimulatorPacket, UiPacket},
    persistence::{self, SaveBuilder},
};
use std::{
    sync::mpsc::TryRecvError,
    time::{Duration, Instant},
};
use threadpool::ThreadPool;

#[cfg(debug_assertions)]
use debug_data::DebugValues;

/// The egui id for the board where the cells are being displayed.
const BOARD_ID: &str = "board";
/// The egui id for the top panel.
const TOP_PANEL: &str = "Top_Panel";
/// The egui id for the settings panel.
pub(crate) const SETTINGS_PANEL: &str = "Settings_Panel";
/// The egui id for the debug window.
#[cfg(debug_assertions)]
const DEBUG_WINDOW: &str = "Debug_Window";
/// The egui id for the edit mode selection.
const EDIT_MODE_SELECT: &str = "Edit Mode";
/// The egui id for the layer the selection is being drawn on.
const SELECTION_LAYER: &str = "SELECTION_LAYER";

lang! {
    EDIT_PREVIEW, "Edit Mode:"
}

/// The struct that contains the data for the gui of my app.
pub struct MyApp<'a> {
    /// Stores extra information used in debug mode.
    #[cfg(debug_assertions)]
    debug: DebugValues,

    /// Stores relevant information for unrecoverable errors.
    error_occurred: Option<ErrorData>,

    /// The updated display produced by the simulator.
    display_update: SharedDisplay,
    /// The current display being rendered.
    display_cache: BoardDisplay,
    /// The area of the board to request being displayed.
    ///
    /// When sending a request to the simulator, ensure to increase the max x by 1 and the min y by -1.
    /// This is due to smooth scrolling requiring an extra tile in each axis.
    display_area: Area,
    /// The x offset from the board being displayed.
    x_offset: f32,
    /// The y offset from the board being displayed.
    y_offset: f32,

    /// A channel to send data to the simulator.
    ui_sender: UiSender,
    /// A channel to receive data from the simulator.
    simulator_receiver: SimulatorReceiver,

    /// The menu & options for saving files.
    save: Save,
    /// The menu & options for loading files.
    load: LoadBoard,

    /// The menu & options for saving blueprints.
    blueprint_save: BlueprintSave,
    /// The menu & options for loading blueprints.
    blueprint_load: BlueprintLoad,

    /// The persistent settings.
    settings: Settings,
    /// Background threads for executing IO operations.
    io_thread: &'a ThreadPool,
    /// Used for spawning toasts.
    toasts: Toasts,

    /// The recent edits the user made to the board.
    history: History,

    /// The current edit mode the user is in
    edit_state: EditState,
    /// The area the user has selected
    selection: Option<Selection>,
}

impl eframe::App for MyApp<'_> {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        #[cfg(debug_assertions)]
        let start_time = Instant::now();
        #[cfg(debug_assertions)]
        self.debug_window(ctx, frame);

        self.settings.interface.apply_style(ctx);

        self.toasts.show(ctx);

        let mut to_send = Vec::new();

        if let Some(error_data) = &mut self.error_occurred {
            handle_error(ctx, error_data);

            // Don't perform any other actions as the application is in an invalid state.
            return;
        }

        to_send.append(&mut self.check_keybinds(ctx));

        self.save.update(ctx, &mut self.settings, &mut self.toasts);
        self.load.update(
            self.io_thread,
            &self.settings.file.save_location,
            &mut self.toasts,
        );

        self.blueprint_save
            .update(self.selection, &mut to_send, &mut self.toasts);
        self.blueprint_load.update(
            self.io_thread,
            &self.settings.file.blueprint_location,
            &mut self.toasts,
        );

        self.save.draw(ctx, &mut to_send);
        self.load.draw(ctx);

        self.blueprint_save.draw(ctx);
        self.blueprint_load.draw(ctx);

        // Stores the size the board will take up.
        let mut board_rect = Rect::from_min_max(
            (0.0, 0.0).into(),
            ctx.input(|i| i.screen_rect()).right_bottom(),
        );

        // Draw settings menu
        if let Some(inner_response) = self.settings.draw(ctx) {
            let size = inner_response.response.rect.size();
            *board_rect.left_mut() += size.x;
        };

        let show = egui::TopBottomPanel::top(TOP_PANEL).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Start").clicked() {
                    to_send.push(UiPacket::Start);
                    self.history.clear();
                };
                if ui.button("Stop").clicked() {
                    to_send.push(UiPacket::Stop);
                }

                if ui.button("Settings").clicked() {
                    self.settings.open = !self.settings.open;
                }

                if ui.button("Save").clicked() {
                    self.save.show = !self.save.show;
                }

                if ui.button("Load").clicked() {
                    self.load.show = !self.load.show
                }

                if ui
                    .add_enabled(self.history.can_undo(), egui::Button::new("Undo"))
                    .clicked()
                {
                    to_send.append(&mut self.history.undo());
                }

                if ui
                    .add_enabled(self.history.can_redo(), egui::Button::new("Redo"))
                    .clicked()
                {
                    to_send.append(&mut self.history.redo());
                }

                egui::ComboBox::from_id_salt(EDIT_MODE_SELECT)
                    .selected_text(format!("{EDIT_PREVIEW} {}", self.edit_state))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.edit_state,
                            EditState::Preview,
                            EditState::Preview.to_string(),
                        );

                        ui.selectable_value(
                            &mut self.edit_state,
                            EditState::Draw,
                            EditState::Draw.to_string(),
                        );

                        ui.selectable_value(
                            &mut self.edit_state,
                            EditState::Select,
                            EditState::Select.to_string(),
                        );
                    });

                if ui.button("Blueprint Save").clicked() {
                    self.blueprint_save.show = !self.blueprint_save.show;
                }

                if ui.button("Blueprint Load").clicked() {
                    self.blueprint_load.show = !self.blueprint_load.show;
                }

                #[cfg(debug_assertions)]
                {
                    if ui.button("Debug Menu").clicked() {
                        self.debug.debug_menu_open = !self.debug.debug_menu_open
                    }
                }
            })
        });

        let top_size = show.response.rect.size();

        // Account for top panel.
        *board_rect.top_mut() += top_size.y;
        *board_rect.bottom_mut() += top_size.y;

        // board_rect must not change after this point
        let board_rect = board_rect;

        self.board_interaction(ctx, &mut to_send, board_rect);
        self.scroll_board(&mut to_send);
        self.draw_board(ctx, board_rect);
        self.draw_selection(ctx, board_rect);

        // Load selected board
        if let Some(save_preview) = self.load.preview_to_load() {
            let mut save_location = self.settings.file.save_location.clone();
            let filename = save_preview.get_filename();

            save_location.push(filename);

            match persistence::load_board_data(save_location.as_path()) {
                Ok(save) => {
                    to_send.push(UiPacket::LoadBoard { board: save });
                    self.toasts.add(
                        Toast::new()
                            .kind(egui_toast::ToastKind::Success)
                            .options(toast_options())
                            .text(format!(
                                "Successfully loaded save \"{}\"",
                                save_preview.get_name()
                            )),
                    );
                }
                Err(err) => {
                    self.toasts.add(
                        Toast::new()
                            .kind(egui_toast::ToastKind::Error)
                            .options(toast_options())
                            .text(format!("Unable to load save file: {err}")),
                    );
                }
            };
        }

        // Load selected board
        if let Some(blueprint_preview) = self.blueprint_load.preview_to_load() {
            let mut blueprint_location = self.settings.file.blueprint_location.clone();
            let filename = blueprint_preview.get_filename();

            blueprint_location.push(filename);

            match persistence::load_blueprint(blueprint_location.as_path()) {
                Ok(blueprint) => {
                    to_send.push(UiPacket::LoadBlueprint {
                        load_position: (0, 0).into(),
                        blueprint,
                    });
                    self.toasts.add(
                        Toast::new()
                            .kind(egui_toast::ToastKind::Success)
                            .options(toast_options())
                            .text(format!(
                                "Successfully loaded blueprint \"{}\"",
                                blueprint_preview.get_name()
                            )),
                    );
                }
                Err(err) => {
                    self.toasts.add(
                        Toast::new()
                            .kind(egui_toast::ToastKind::Error)
                            .options(toast_options())
                            .text(format!("Unable to load blueprint: {err}")),
                    );
                }
            };
        }

        // Process fallible code //

        // Update display
        match self.display_update.try_lock() {
            Ok(mut board) => {
                if let Some(board) = board.take() {
                    // Allow updates to be paused for debug testing.
                    #[cfg(debug_assertions)]
                    if self.debug.update_board {
                        self.display_cache = board;
                    }

                    // Always update when not in debug build
                    #[cfg(not(debug_assertions))]
                    {
                        self.display_cache = board;
                    }
                }
            }
            Err(std::sync::TryLockError::WouldBlock) => {
                // The display cache can still be used.
            }
            Err(std::sync::TryLockError::Poisoned(err)) => {
                self.error_occurred = Some(ErrorData::from_error_and_log(
                    lang::SHARED_DISPLAY_POISIONED,
                    err,
                ));
                return;
            }
        }

        // Process user interaction
        for message in to_send {
            if let Err(err) = self.ui_sender.send(message) {
                self.error_occurred = Some(ErrorData::from_error_and_log(lang::SEND_ERROR, err));
                return;
            }
        }

        loop {
            // Receive packets from simulatior
            let simulator_packet = match self.simulator_receiver.try_recv() {
                Ok(simulator_packet) => simulator_packet,
                Err(TryRecvError::Empty) => {
                    break;
                }
                Err(TryRecvError::Disconnected) => {
                    self.error_occurred = Some(ErrorData::from_error(lang::RECEIVE_ERROR));
                    return;
                }
            };

            // Act on the simulator packets
            match simulator_packet {
                SimulatorPacket::BoardSave {
                    board: simulation_save,
                } => {
                    let name = self.save.get_name().to_owned();
                    let description = self.save.get_description().to_owned();
                    let mut tags = self.save.get_tags().clone();
                    let save_path = self.settings.file.save_location.clone();

                    // Convert tags
                    let tags = tags
                        .iter_mut()
                        .map(|tag| tag.clone().into_boxed_str())
                        .collect();

                    let (tx, rx) = oneshot::channel();

                    self.save.set_waiting(rx);
                    // Run task in IO thread
                    self.io_thread.execute(move || {
                        let _ = tx
                            .send(
                                SaveBuilder::new_save(simulation_save)
                                    .name(name)
                                    .desciprtion(description)
                                    .tags(tags)
                                    .save(save_path),
                            )
                            .inspect_err(|e| {
                                eprintln!("Could not communicate with ui thread: {e}")
                            });
                    });
                }
                SimulatorPacket::BlueprintSave { blueprint } => self.blueprint_save.save_blueprint(
                    blueprint,
                    self.io_thread,
                    self.settings.file.blueprint_location.clone(),
                ),
            }
        }

        // Time framerate
        #[cfg(debug_assertions)]
        {
            let end_time = Instant::now();
            self.debug.last_frame_time = end_time - start_time;
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, Settings::SAVE_KEY, &self.settings);
    }
}

impl<'a> MyApp<'a> {
    pub fn new(
        creation_context: &eframe::CreationContext<'_>,
        display: SharedDisplay,
        ui_sender: UiSender,
        simulator_receiver: SimulatorReceiver,
        io_thread: &'a ThreadPool,
    ) -> Self {
        let mut my_app = MyApp {
            display_update: display,
            display_cache: Default::default(),
            ui_sender,
            simulator_receiver,
            error_occurred: None,
            x_offset: 0.0,
            y_offset: 0.0,
            display_area: Area::new((-10, -10), (10, 10)),
            settings: Settings::default(),
            save: Save::default(),
            load: Default::default(),
            io_thread,
            toasts: Toasts::new(),
            history: Default::default(),
            edit_state: Default::default(),
            selection: None,
            #[cfg(debug_assertions)]
            debug: DebugValues::default(),
            blueprint_save: Default::default(),
            blueprint_load: Default::default(),
        };

        // Load stored configurations
        if let Some(storage) = creation_context.storage {
            if let Some(settings) = eframe::get_value(storage, Settings::SAVE_KEY) {
                my_app.settings = settings;
            };
        }

        my_app
            .ui_sender
            .send(UiPacket::Set {
                position: (0, 0).into(),
                cell_state: Cell::Alive,
            })
            .unwrap();

        my_app
            .ui_sender
            .send(UiPacket::Set {
                position: (0, 1).into(),
                cell_state: Cell::Alive,
            })
            .unwrap();

        my_app
            .ui_sender
            .send(UiPacket::Set {
                position: (0, 2).into(),
                cell_state: Cell::Alive,
            })
            .unwrap();

        my_app
    }

    /// Draws the debug window.
    ///
    /// This method only exists on debug builds.
    #[cfg(debug_assertions)]
    fn debug_window(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::Window::new(DEBUG_WINDOW)
            .open(&mut self.debug.debug_menu_open)
            .show(ctx, |ui| {
                ui.heading("Errors");
                ui.horizontal_top(|ui| {
                    if ui
                        .button("Cause error")
                        .on_hover_text("Tests the unrecoverable error feature")
                        .clicked()
                    {
                        self.error_occurred = Some(ErrorData::from_error(
                            "Test error occurred! Remove it with the debug menu.",
                        ));
                    }

                    if ui
                        .button("Clear error")
                        .on_hover_text(
                            "Clears the current unrecoverable error\n⚠ Use with caution! ⚠",
                        )
                        .clicked()
                    {
                        self.error_occurred = None;
                    }
                });

                ui.separator();
                ui.heading("Internal Values");

                ui.label(format!(
                    "Error Occurred: {}",
                    match &self.error_occurred {
                        Some(err) => format!("{:?}", err),
                        None => "No Error".to_owned(),
                    }
                ));

                ui.horizontal(|ui| {
                    ui.label(format!("X Offset: {:.2}", self.x_offset));
                    ui.add(
                        egui::DragValue::new(&mut self.x_offset)
                            // Will cap the scroll offset if the range is not dynamic
                            .range(0..=self.settings.cell.size as i32)
                            .speed(0.1),
                    );
                });

                ui.horizontal(|ui| {
                    ui.label(format!("Y Offset: {:.2}", self.y_offset));
                    ui.add(
                        egui::DragValue::new(&mut self.y_offset)
                            // Will cap the scroll offset if the range is not dynamic
                            .range(0..=self.settings.cell.size as i32)
                            .speed(0.1),
                    );
                });

                ui.label(format!(
                    "Generation: {}",
                    self.display_cache.get_generation()
                ));

                ui.heading("Cursor Positions:");
                ui.label(format!(
                    "Egui Position: {}",
                    match ctx.pointer_latest_pos() {
                        Some(pos) => pos.to_string(),
                        None => "Offscreen".to_owned(),
                    },
                ));

                ui.label(format!(
                    "Local Board Position: {}",
                    match ctx.pointer_latest_pos() {
                        Some(position) => {
                            let x = position.x - self.x_offset;
                            let y = position.y - self.y_offset;

                            // Position of cell
                            let local_x = (x / self.settings.cell.size).trunc() as i32;
                            let local_y = (y / self.settings.cell.size).trunc() as i32;

                            format!("{:#?}", GlobalPosition::new(local_x, local_y))
                        }
                        None => "Offscreen".to_owned(),
                    }
                ));

                ui.separator();
                ui.heading("Rendering Stats");

                let update_duration = self.debug.last_frame_time.as_secs_f32();
                let updates_per_sec = 1.0 / update_duration;
                ui.label(format!(
                    "Updates Per Second: {}",
                    updates_per_sec.to_string()
                ));

                let cpu_usage = frame
                    .info()
                    .cpu_usage
                    .map(|usage| usage.to_string())
                    .unwrap_or("N/A".to_owned());
                ui.label(format!("CPU Usage: {cpu_usage}"));

                ui.separator();
                if ui.button("Spawn Toast").clicked() {
                    self.toasts.add(
                        Toast::new()
                            .kind(egui_toast::ToastKind::Info)
                            .options(toast_options())
                            .text("Testing toasts!"),
                    );
                }

                ui.separator();
                ui.checkbox(&mut self.debug.update_board, "Update board from simulator?");

                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Focused:");
                    ui.label(
                        ctx.memory(|memory| memory.focused())
                            .map_or("None".to_string(), |id| id.short_debug_format()),
                    );
                });
            });
    }

    /// Checks if any keybinds have been pressed & executes the corresponding action.
    fn check_keybinds(&mut self, ctx: &egui::Context) -> Vec<UiPacket> {
        let mut to_send = Vec::new();

        // If the user is typing don't allow keybinds.
        if ctx.wants_keyboard_input() {
            return to_send;
        }

        ctx.input_mut(|input| {
            for keybind in self.settings.keybind.pressed(input) {
                match keybind {
                    Keybind::SettingsMenu => self.settings.open = !self.settings.open,
                    Keybind::StartSimulation => to_send.push(UiPacket::Start),
                    Keybind::StopSimulation => to_send.push(UiPacket::Stop),
                    Keybind::LoadBoard => self.load.show = !self.load.show,
                    Keybind::LoadBlueprint => self.blueprint_load.show = !self.blueprint_load.show,
                    Keybind::SaveBoard => self.save.show = !self.save.show,
                    Keybind::SaveBlueprint => self.blueprint_save.show = !self.blueprint_save.show,
                }
            }
        });

        to_send
    }

    /// Draws the board of for Conways Game of Life onto the centeral panel.
    fn draw_board(&mut self, ctx: &egui::Context, board_rect: Rect) {
        // Creates the painter for the board display.
        let layer_painter = Painter::new(
            ctx.clone(), // ctx is cloned in egui implementations.
            egui::LayerId::new(egui::Order::Background, BOARD_ID.into()),
            board_rect,
        );

        // Draw the background as the cell dead colour by default
        layer_painter.add(RectShape::filled(
            board_rect,
            CornerRadius::ZERO,
            self.settings.cell.dead_colour,
        ));

        // Number of cell in x axis
        let x_cells = (board_rect.right() / self.settings.cell.size).ceil() as u32;
        // Create iterator of x position for cells.
        // An extra cell is generated to compensate for the scroll offset.
        let x_iter = (0..x_cells + 1).map(|x| {
            let mut x_cell = x as f32;
            x_cell *= self.settings.cell.size;
            // Offset by the scroll position.
            x_cell -= self.settings.cell.size - self.x_offset;
            x_cell
        });

        // Number of cells in y axis
        let y_cells = (board_rect.bottom() / self.settings.cell.size).floor() as u32;
        // Create iterator of y position for cells.
        // An extra cell is generated to compensate for the scroll offset.
        let y_iter = (0..y_cells + 1).map(|y| {
            let mut y_cell = y as f32;
            y_cell *= self.settings.cell.size;
            // Offset by the scroll position.
            y_cell -= self.settings.cell.size - self.y_offset;
            y_cell
        });

        // Ensure that the size of the display area is the same as the number of cells displayed.
        self.display_area.modify_max((
            // Subtract needs to be performed as i32, because the result might be negative.
            (x_cells as i32 - self.display_area.x_difference() as i32),
            (y_cells as i32 - self.display_area.y_difference() as i32),
        ));

        // Uses the difference to offset the cells being rendered.
        // This allows the cells to move on the board without the simulator sending new data.
        let x_diff =
            self.display_area.get_max().get_x() - self.display_cache.get_area().get_max().get_x();
        let y_diff =
            self.display_area.get_max().get_y() - self.display_cache.get_area().get_max().get_y();

        // Draws the alive cells.
        for (x_index, x_origin) in x_iter.clone().enumerate() {
            for (y_index, y_origin) in y_iter.clone().enumerate() {
                if let Cell::Alive = self
                    .display_cache
                    .get_cell((x_index as i32 + x_diff, y_index as i32 + y_diff))
                {
                    let rect = Rect::from_two_pos(
                        pos2(x_origin, y_origin),
                        pos2(
                            x_origin + self.settings.cell.size,
                            y_origin + self.settings.cell.size,
                        ),
                    );

                    let rect = RectShape::filled(
                        rect,
                        CornerRadius::ZERO,
                        self.settings.cell.alive_colour,
                    );

                    layer_painter.add(rect);
                };
            }
        }

        // Draw the lines last so that they draw over the cells.

        // Draws the vertical lines.
        for x_offset in x_iter {
            layer_painter.vline(
                x_offset,
                board_rect.y_range(),
                egui::Stroke::new(1.0, self.settings.cell.grid_colour),
            );
        }

        // Draws the horizontal lines.
        for y_offset in y_iter {
            layer_painter.hline(
                board_rect.x_range(),
                y_offset,
                egui::Stroke::new(1.0, self.settings.cell.grid_colour),
            );
        }
    }

    /// Process interactions for the board.
    fn board_interaction(
        &mut self,
        ctx: &egui::Context,
        to_send: &mut Vec<UiPacket>,
        board_rect: Rect,
    ) {
        // Draws the central panel to provide the area for user interaction.
        let interact = egui::CentralPanel::default()
            .show(ctx, |ui| {
                ui.interact(
                    board_rect,
                    Id::new("Board_Drag_Sense"),
                    egui::Sense::click_and_drag(),
                )
            })
            .inner;

        match self.edit_state {
            EditState::Preview => preview_interaction(
                &mut self.x_offset,
                &mut self.y_offset,
                &mut self.display_area,
                self.settings.cell.size,
                to_send,
                interact,
            ),
            EditState::Draw => draw_interaction(
                self.settings.cell.size,
                self.display_area,
                &self.display_cache,
                self.x_offset,
                self.y_offset,
                &mut self.history,
                to_send,
                interact,
            ),
            EditState::Select => {
                // Create a new selection when a new drag is started.
                if let (true, Some(pointer_position)) =
                    (interact.drag_started(), self.global_position(ctx))
                {
                    self.selection = Some(Selection::new(pointer_position));
                }

                // Handle selection interaction
                if let Some(ref mut selection) = self.selection {
                    select_interaction(
                        ctx,
                        interact,
                        selection,
                        self.settings.cell.size,
                        self.display_area.get_min(),
                    );
                }
            }
        }
    }

    /// Draws the current selection ontop of the board.
    fn draw_selection(&mut self, ctx: &egui::Context, board_rect: Rect) {
        if let Some(ref mut selection) = self.selection {
            // Setup
            let layer_id = egui::LayerId::new(egui::Order::Background, SELECTION_LAYER.into());
            let (pos1, pos2) = selection.get_draw_positions(
                self.display_area.get_min(),
                self.settings.cell.size,
                self.x_offset,
                self.y_offset,
            );
            let rect = egui::Rect::from_two_pos(pos1, pos2);

            // Draw
            let painter = egui::Painter::new(ctx.clone(), layer_id, board_rect);
            let rect_shape = RectShape::stroke(
                rect,
                1.0,
                egui::Stroke::new(5.0, self.settings.cell.selection_colour),
                egui::StrokeKind::Middle,
            );
            painter.add(rect_shape);
        }
    }

    fn scroll_board(&mut self, to_send: &mut Vec<UiPacket>) {
        // While loops are used as display can be dragged further than one cell in one frame.
        while self.x_offset % self.settings.cell.size > 0.0 {
            self.display_area.translate_x(-1);
            self.x_offset -= self.settings.cell.size;
        }

        while self.x_offset % self.settings.cell.size < 0.0 {
            self.display_area.translate_x(1);
            self.x_offset += self.settings.cell.size;
        }

        while self.y_offset % self.settings.cell.size > 0.0 {
            self.display_area.translate_y(-1);
            self.y_offset -= self.settings.cell.size;
        }

        while self.y_offset % self.settings.cell.size < 0.0 {
            self.display_area.translate_y(1);
            self.y_offset += self.settings.cell.size;
        }

        // Check if the displayed data is different to the displayed area.
        if self.display_area != self.display_cache.get_area() {
            let mut new_area = self.display_area;

            new_area.modify_max((0, 1));
            new_area.modify_min((-1, 0));

            to_send.push(UiPacket::DisplayArea { new_area });
        }
    }

    /// Gets the global position on the board that the current pointer interaction is occurring on.
    fn global_position(&self, ctx: &egui::Context) -> Option<GlobalPosition> {
        let cell_size = self.settings.cell.size;
        let display_area = self.display_area;
        let position = ctx.pointer_interact_pos()?;

        // Position of cell
        let x = position.x + self.x_offset;
        let y = position.y + self.y_offset;

        let local_x = (x / cell_size).trunc() as i32;
        let local_y = (y / cell_size).trunc() as i32;

        // Position of displayed board
        let offset_x = display_area.get_min().get_x();
        let offset_y = display_area.get_min().get_y();

        let position = GlobalPosition::new(local_x + offset_x, local_y + offset_y);
        Some(position)
    }

    /// Gets the local position on the board that the current pointer interaction is occurring on.
    fn local_position(&self, ctx: &egui::Context) -> Option<GlobalPosition> {
        let cell_size = self.settings.cell.size;
        let position = ctx.pointer_interact_pos()?;

        // Position of cell
        let x = position.x + self.x_offset;
        let y = position.y + self.y_offset;

        let local_x = (x / cell_size).trunc() as i32;
        let local_y = (y / cell_size).trunc() as i32;

        Some(GlobalPosition::new(local_x, local_y))
    }
}

/// Draws the fatial error in the middle of the screen.
fn handle_error(ctx: &egui::Context, error_data: &mut ErrorData) {
    // Ensures the background is empty.
    egui::CentralPanel::default().show(ctx, |_ui| {});

    // Calculates the position of the window.
    let screen_center = ctx.screen_rect().center();
    let position = error_data
        .window_size
        .map(|size| {
            let x_offset = size.x / 2.0;
            let y_offset = size.y / 2.0;

            let x = screen_center.x - x_offset;
            let y = screen_center.y - y_offset;

            egui::pos2(x, y)
        })
        .unwrap_or(screen_center);

    // Create pop-up window to display error.
    // Centering normal text is a nightmare so a pop-up will sufice.
    let window = egui::Window::new(lang::UNRECOVERABLE_ERROR_HEADER)
        .movable(false)
        .order(egui::Order::Foreground)
        .current_pos(position)
        .resizable(false)
        .show(ctx, |ui| {
            ui.label(format!(
                "{}\"{}\"",
                lang::ERROR_MESSAGE,
                error_data.error_message
            ));
            ui.label(lang::ERROR_ADVICE)
        });

    // Calculate the current size of the pop-up to use for centering on the next frame.
    if let Some(window) = window {
        error_data.window_size = Some(window.response.rect.size());
    }
}

/// Stores relevant information for unrecoverable errors.
#[cfg_attr(debug_assertions, derive(Debug))]
struct ErrorData {
    /// The error message.
    error_message: &'static str,
    /// The size of the window displaying the error the previous frame.
    ///
    /// This is used to centre the window.
    window_size: Option<egui::Vec2>,
}

impl ErrorData {
    /// Creates a new [`ErrorData`] with the given sing as the error message.
    pub fn from_error(error_message: &'static str) -> Self {
        ErrorData {
            error_message,
            window_size: None,
        }
    }

    /// Create a new [`ErrorData`] with the given string as the error message; Outputting the given error as a
    /// standardised log message.
    pub fn from_error_and_log(error_message: &'static str, error: impl std::error::Error) -> Self {
        log::error!("{} - {}", error_message, error);
        Self::from_error(error_message)
    }
}

/// Generates default toast options.
pub(crate) fn toast_options() -> egui_toast::ToastOptions {
    egui_toast::ToastOptions::default()
        .duration_in_seconds(2.0)
        .show_progress(true)
        .show_icon(true)
}

/// Contains the values used during the application debugging.
#[cfg(debug_assertions)]
mod debug_data {
    use std::time::Duration;

    /// Contains the values used during the application debugging.
    pub(crate) struct DebugValues {
        /// Whether the debug window is open or not.
        pub(crate) debug_menu_open: bool,
        /// Time since last frame.
        pub(crate) last_frame_time: Duration,
        /// Whether to stop the board from being updated from the simulator.
        pub(crate) update_board: bool,
    }

    impl Default for DebugValues {
        fn default() -> Self {
            Self {
                debug_menu_open: true,
                last_frame_time: Duration::new(0, 0),
                update_board: true,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // use egui_kittest::kittest::Queryable;

    // #[test]
    // fn default_view() {
    //     let ((ui_sender, ui_receiver), (simulator_sender, simulator_receiver)) =
    //         gol_lib::create_channels();

    //     // Start IO thread.
    //     let io_threads = threadpool::Builder::new()
    //         .num_threads(1)
    //         .thread_name("Background IO thread".to_owned())
    //         .build();

    //     // Start app
    //     let mut harness = egui_kittest::HarnessBuilder::default().build_eframe(|cc| {
    //         MyApp::new(
    //             cc,
    //             Default::default(),
    //             ui_sender.clone(),
    //             simulator_receiver,
    //             &io_threads,
    //         )
    //     });

    //     // Close the debug window.
    //     harness
    //         .get_by_role_and_label(egui::accesskit::Role::Window, "Debug_Window")
    //         .get_by_role_and_label(egui::accesskit::Role::Button, "Close window")
    //         .click();

    //     // The window takes two frames to close.
    //     harness.step();
    //     harness.step();

    //     // Take a screenie
    //     harness.snapshot("default_window");
    // }
}
