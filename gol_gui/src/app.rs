#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use crate::{
    file_management::{Load, Save},
    lang,
    settings::Settings,
    USER_SAVE_PATH,
};
use egui::{pos2, Color32, Id, Painter, Rect};
use egui_keybind::Bind;
use gol_lib::{
    board_data::BoardSave,
    communication::{SimulatorPacket, UiPacket},
    Area, BoardDisplay, Cell, GlobalPosition, SharedDisplay, SimulatorReceiver, UiSender,
};
use std::{
    sync::mpsc::TryRecvError,
    time::{Duration, Instant},
};

/// The egui id for the board where the cells are being displayed.
const BOARD_ID: &str = "board";
/// The egui id for the top panel.
const TOP_PANEL: &str = "Top_Panel";
/// The egui id for the settings panel.
pub(crate) const SETTINGS_PANEL: &str = "Settings_Panel";
/// The egui id for the debug window.
#[cfg(debug_assertions)]
const DEBUG_WINDOW: &str = "Debug_Window";

/// The struct that contains the data for the gui of my app.
pub struct MyApp<'a> {
    label: &'a str,

    /// Whether the debug window is open or not.
    #[cfg(debug_assertions)]
    debug_menu_open: bool,
    /// Time since last frame.
    #[cfg(debug_assertions)]
    last_frame_time: Duration,

    /// Stores relevant information for unrecoverable errors.
    error_occurred: Option<ErrorData>,

    /// The updated display produced by the simulator.
    display_update: SharedDisplay,
    /// The current display being rendered.
    display_cache: BoardDisplay,
    /// The area of the board to request being displayed.
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
    load: Load,

    /// The persistent settings.
    settings: Settings,
}

impl MyApp<'static> {
    pub fn new(
        creation_context: &eframe::CreationContext<'_>,
        display: SharedDisplay,
        ui_sender: UiSender,
        simulator_receiver: SimulatorReceiver,
    ) -> Self {
        let mut my_app = MyApp {
            label: "Hello world",
            display_update: display,
            display_cache: Default::default(),
            ui_sender,
            simulator_receiver,
            error_occurred: None,
            #[cfg(debug_assertions)]
            debug_menu_open: true,
            x_offset: 0.0,
            y_offset: 0.0,
            display_area: Area::new((-10, -10), (10, 10)),
            #[cfg(debug_assertions)]
            last_frame_time: Duration::new(0, 0),
            settings: Settings::default(),
            save: Save::default(),
            load: Default::default(),
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
            .ui_sender
            .send(UiPacket::DisplayArea {
                new_area: my_app.display_area,
            })
            .unwrap();

        my_app
    }

    /// Draws the debug window.
    ///
    /// This method only exists on debug builds.
    #[cfg(debug_assertions)]
    fn debug_window(&mut self, ctx: &egui::Context) {
        egui::Window::new(DEBUG_WINDOW)
            .open(&mut self.debug_menu_open)
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
                // ui.add(egui::Separator::horizontal())
                ui.separator();
                ui.heading("Internal Values");
                ui.label(format!(
                    "Error Occurred: {}\n\
                        Display Area: {:?}\n\
                        X Offset: {}\n\
                        Y Offset: {}\n\
                        Cell Alive Colour: {:#?}\n\
                        Cell Dead Colour: {:#?}\n\
                        Cell Size: {}",
                    match &self.error_occurred {
                        Some(err) => format!("{:?}", err),
                        None => "No Error".to_owned(),
                    },
                    self.display_area,
                    self.x_offset,
                    self.y_offset,
                    self.settings.cell.alive_colour,
                    self.settings.cell.dead_colour,
                    self.settings.cell.size
                ));
                ui.label(format!(
                    "Cursor Position: {}",
                    match ctx.pointer_latest_pos() {
                        Some(pos) => pos.to_string(),
                        None => "Offscreen".to_owned(),
                    },
                ));

                ui.separator();
                let secs_f64 = self.last_frame_time.as_secs_f64();
                if secs_f64.is_normal() {
                    let fps = 1.0 / secs_f64;
                    ui.label(fps.to_string());
                }
            });
    }

    /// Checks if any keybinds have been pressed & executes the corresponding action.
    fn check_keybinds(&mut self, ctx: &egui::Context) {
        let keybind = &mut self.settings.keybind;

        ctx.input_mut(|input| {
            if keybind.settings_menu.pressed(input) {
                self.settings.open = !self.settings.open;
            }
        })
    }
}

impl eframe::App for MyApp<'static> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        #[cfg(debug_assertions)]
        let start_time = Instant::now();
        #[cfg(debug_assertions)]
        self.debug_window(ctx);

        let mut to_send = Vec::new();

        if let Some(error_data) = &mut self.error_occurred {
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

            // Don't perform any other actions as the application is in an invalid state.
            return;
        }

        self.check_keybinds(ctx);

        self.save.draw(ctx, &mut to_send);
        self.load.draw(ctx);

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

                #[cfg(debug_assertions)]
                {
                    if ui.button("Debug Menu").clicked() {
                        self.debug_menu_open = !self.debug_menu_open
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

        // Draws the central panel to provide the area for user interaction.
        egui::CentralPanel::default().show(ctx, |ui| {
            let interact = ui.interact(
                board_rect,
                Id::new("Board_Drag_Sense"),
                egui::Sense::click_and_drag(),
            );

            // Scroll the display in response to user dragging mouse
            if interact.dragged() {
                let drag_delta = interact.drag_delta();
                self.x_offset += drag_delta.x;
                self.y_offset += drag_delta.y;

                let mut modified_display = false;

                // While loops are used as display can be dragged further than one cell in one frame.
                while self.x_offset % self.settings.cell.size > 0.0 {
                    self.display_area.translate_x(-1);
                    self.x_offset -= self.settings.cell.size;
                    modified_display = true;
                }

                while self.x_offset % self.settings.cell.size < 0.0 {
                    self.display_area.translate_x(1);
                    self.x_offset += self.settings.cell.size;
                    modified_display = true;
                }

                while self.y_offset % self.settings.cell.size > 0.0 {
                    self.display_area.translate_y(-1);
                    self.y_offset -= self.settings.cell.size;
                    modified_display = true;
                }

                while self.y_offset % self.settings.cell.size < 0.0 {
                    self.display_area.translate_y(1);
                    self.y_offset += self.settings.cell.size;
                    modified_display = true;
                }

                if modified_display {
                    to_send.push(UiPacket::DisplayArea {
                        new_area: self.display_area,
                    });
                }
            }

            // Toggles the state of a cell when it is clicked.
            if interact.clicked() {
                if let Some(position) = interact.interact_pointer_pos() {
                    // Position of cell
                    let cell_x = (position.x / self.settings.cell.size).trunc() as i32;
                    let cell_y = (position.y / self.settings.cell.size).trunc() as i32;

                    // Position of displayed board
                    let origin_x = self.display_area.get_min().get_x();
                    let origin_y = self.display_area.get_min().get_y();

                    let position = GlobalPosition::new(cell_x + origin_x, cell_y + origin_y);
                    let cell_state = self.display_cache.get_cell((cell_x, cell_y)).invert();
                    to_send.push(UiPacket::Set {
                        position,
                        cell_state,
                    });
                }
            }
        });

        // Creates the painter for the board display.
        let layer_painter = Painter::new(
            ctx.clone(), // ctx is cloned in egui implementations.
            egui::LayerId::new(egui::Order::Background, BOARD_ID.into()),
            board_rect,
        );

        // Number of cell in x axis
        let x_cells = (board_rect.right() / self.settings.cell.size).ceil() as i32;
        // Create iterator of x position for cells
        let x_iter = (0..x_cells).map(|x| {
            let mut x_cell = x as f32;
            x_cell *= self.settings.cell.size;
            x_cell
        });

        // Number of cells in y axis
        let y_cells = (board_rect.bottom() / self.settings.cell.size).floor() as i32;
        // Create iterator of y position for cells
        let y_iter = (0..y_cells).map(|y| {
            let mut y_cell = y as f32;
            y_cell *= self.settings.cell.size;
            y_cell
        });

        // Modify displayed area to follow cells displayed.
        self.display_area
            .modify_x(x_cells - self.display_area.x_difference());
        self.display_area
            .modify_y(y_cells - self.display_area.y_difference());

        // Draw the display board.
        for (x_index, x_origin) in x_iter.enumerate() {
            for (y_index, y_origin) in y_iter.clone().enumerate() {
                let rect = Rect::from_two_pos(
                    pos2(x_origin, y_origin),
                    pos2(
                        x_origin + self.settings.cell.size,
                        y_origin + self.settings.cell.size,
                    ),
                );

                let rect = egui::epaint::RectShape::new(
                    rect,
                    egui::Rounding::ZERO,
                    {
                        match self
                            .display_cache
                            .get_cell((x_index as i32, y_index as i32))
                        {
                            Cell::Alive => self.settings.cell.alive_colour,
                            Cell::Dead => self.settings.cell.dead_colour,
                        }
                    },
                    egui::Stroke::new(1.0, Color32::GRAY),
                );

                layer_painter.add(rect);
            }
        }

        // If update is not requested the board will become outdated.
        // This causes higher cpu usage, but only by one/two %.
        ctx.request_repaint();

        // Process fallible code //

        // Update display
        match self.display_update.try_lock() {
            Ok(mut board) => {
                if let Some(board) = board.take() {
                    self.display_cache = board;
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

            match simulator_packet {
                SimulatorPacket::BoardSave { board } => {
                    BoardSave::new(
                        self.save.save_name.clone(),
                        self.save.save_description.clone(),
                        None,
                        board,
                    )
                    .save(USER_SAVE_PATH.clone());

                    self.save.save_requested = false;
                }
                SimulatorPacket::BlueprintSave { blueprint } => todo!(),
            }
        }

        // Time framerate
        #[cfg(debug_assertions)]
        {
            let end_time = Instant::now();
            self.last_frame_time = end_time - start_time;
        }
    }

    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, Settings::SAVE_KEY, &self.settings);
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
