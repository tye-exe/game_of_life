#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui::{pos2, Color32, Id, Painter, Rect};

use crate::{
    error_text,
    logic::{
        Area, BoardDisplay, GlobalPosition, SharedDisplay, SimulatorReceiver, UiPacket, UiSender,
    },
};

mod lang {
    use crate::lang;

    lang! {
        APP_NAME, "Game Of Life";
        CELL_SIZE_SLIDER, "Cell Size";
        UNRECOVERABLE_ERROR_HEADER, "Encountered Unrecoverable Error";
        ERROR_MESSAGE, "Error: ";
        ERROR_ADVICE, "Please restart the application.";
        SEND_ERROR, "Unable to send packet to simulation.";
        SHARED_DISPLAY_POISIONED, "Unable to read board from simulation."
    }
}

/// Runs the ui.
pub fn ui_init(
    display: SharedDisplay,
    ui_sender: UiSender,
    simulator_receiver: SimulatorReceiver,
) -> eframe::Result<()> {
    let native_options = eframe::NativeOptions {
        ..Default::default()
    };

    let run_native = eframe::run_native(
        lang::APP_NAME,
        native_options,
        Box::new(|cc| {
            Ok(Box::new(MyApp::new(
                cc,
                display,
                ui_sender.clone(),
                simulator_receiver,
            )))
        }),
    );

    // Command similator thread to terminate after the ui is closed.
    if ui_sender.send(crate::logic::UiPacket::Terminate).is_err() {
        log::error!("{}", error_text::COMMAND_SIM_THREAD_TERM)
    };
    run_native
}

/// The egui id for the board where the cells are being displayed.
const BOARD_ID: &str = "board";
/// The egui id for the top panel.
const TOP_PANEL: &str = "Top_Panel";
/// The egui id for the right panel.
const RIGHT_PANEL: &str = "Right_Panel";
/// The egui id for the debug window.
#[cfg(debug_assertions)]
const DEBUG_WINDOW: &str = "Debug_Window";

/// The struct that contains the data for the gui of my app.
struct MyApp<'a> {
    label: &'a str,

    /// Whether the debug window is open or not.
    #[cfg(debug_assertions)]
    debug_menu_open: bool,

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

    /// The colour of alive cells.
    cell_alive_colour: Color32,
    /// The colour of dead cells.
    cell_dead_colour: Color32,
    /// The size of each cell.
    cell_size: f32,
}

impl MyApp<'static> {
    pub fn new(
        _creation_context: &eframe::CreationContext<'_>,
        display: SharedDisplay,
        ui_sender: UiSender,
        simulator_receiver: SimulatorReceiver,
    ) -> Self {
        let my_app = MyApp {
            label: "Hello world",
            display_update: display,
            display_cache: Default::default(),
            cell_alive_colour: Color32::WHITE,
            cell_dead_colour: Color32::BLACK,
            cell_size: 15.0,
            ui_sender,
            simulator_receiver,
            error_occurred: None,
            #[cfg(debug_assertions)]
            debug_menu_open: true,
            x_offset: 0.0,
            y_offset: 0.0,
            display_area: Area::new((-10, -10), (10, 10)),
        };

        my_app
            .ui_sender
            .send(UiPacket::Set {
                position: (0, 0).into(),
                cell_state: crate::logic::Cell::Alive,
            })
            .unwrap();

        my_app
            .ui_sender
            .send(UiPacket::Set {
                position: (0, 1).into(),
                cell_state: crate::logic::Cell::Alive,
            })
            .unwrap();

        my_app
            .ui_sender
            .send(UiPacket::Set {
                position: (0, 2).into(),
                cell_state: crate::logic::Cell::Alive,
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
}

impl eframe::App for MyApp<'static> {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut to_send = Vec::new();

        #[cfg(debug_assertions)]
        {
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
                        self.cell_alive_colour,
                        self.cell_dead_colour,
                        self.cell_size
                    ));
                    ui.label(format!(
                        "Cursor Position: {}",
                        match ctx.pointer_latest_pos() {
                            Some(pos) => pos.to_string(),
                            None => "Offscreen".to_owned(),
                        },
                    ));
                });
        }

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

        // Stores the size the board will take up.
        let mut board_rect = Rect::from_min_max(
            (0.0, 0.0).into(),
            ctx.input(|i| i.screen_rect()).right_bottom(),
        );

        let show = egui::TopBottomPanel::top(TOP_PANEL).show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Start").clicked() {
                    to_send.push(UiPacket::Start);
                };
                if ui.button("Stop").clicked() {
                    to_send.push(UiPacket::Stop);
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

        // Draws the right side panel & gets the size of it.
        let panel_size = egui::SidePanel::right(RIGHT_PANEL)
            .show(ctx, |ui| {
                ui.add(
                    egui::Slider::new(&mut self.cell_size, 4.0..=50.0)
                        // The slider limits should just be suggestions for the user.
                        .clamping(egui::SliderClamping::Never)
                        .text(lang::CELL_SIZE_SLIDER),
                );
                // However the cells can't be smaller than one pixel as it does not
                // make sense & destroys performance.
                self.cell_size = self.cell_size.max(1.0);
            })
            .response
            .rect
            .size();

        // Reduces the board area to exclude the side panel
        *board_rect.right_mut() -= panel_size.x;

        // Draws the central panel to provide the area for user interaction.
        egui::CentralPanel::default().show(ctx, |ui| {
            let interact = ui.interact(
                board_rect.clone(),
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
                while self.x_offset % self.cell_size > 0.0 {
                    self.display_area.translate_x(-1);
                    self.x_offset -= self.cell_size;
                    modified_display = true;
                }

                while self.x_offset % self.cell_size < 0.0 {
                    self.display_area.translate_x(1);
                    self.x_offset += self.cell_size;
                    modified_display = true;
                }

                while self.y_offset % self.cell_size > 0.0 {
                    self.display_area.translate_y(-1);
                    self.y_offset -= self.cell_size;
                    modified_display = true;
                }

                while self.y_offset % self.cell_size < 0.0 {
                    self.display_area.translate_y(1);
                    self.y_offset += self.cell_size;
                    modified_display = true;
                }

                if modified_display {
                    to_send.push(UiPacket::DisplayArea {
                        new_area: self.display_area,
                    });
                }
            }

            if interact.clicked() {
                // Click logic
            }
        });

        // Creates the painter for the board display.
        let layer_painter = Painter::new(
            ctx.clone(), // ctx is cloned in egui implementations.
            egui::LayerId::new(egui::Order::Background, BOARD_ID.into()),
            board_rect,
        );

        // Iterator over number of x cells in board.
        let x_cells = (board_rect.right() / self.cell_size).ceil() as i32;
        // Create iterator of x origin for cells
        let x_iter = (0..x_cells).map(|x| {
            let mut x_cell = x as f32;
            x_cell *= self.cell_size;
            x_cell
        });

        // Iterator over number of x cells in board.
        let y_cells = (board_rect.bottom() / self.cell_size).floor() as i32;
        // Create iterator of x origin for cells
        let y_iter = (0..y_cells).map(|y| {
            let mut y_cell = y as f32;
            y_cell *= self.cell_size;
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
                    pos2(x_origin as f32, y_origin as f32),
                    pos2(
                        x_origin as f32 + self.cell_size,
                        y_origin as f32 + self.cell_size,
                    ),
                );

                let rect = egui::epaint::RectShape::new(
                    rect,
                    egui::Rounding::ZERO,
                    {
                        match self
                            .display_cache
                            .get_cell((x_index as i32 + 2, y_index as i32 + 2))
                        {
                            crate::logic::Cell::Alive => self.cell_alive_colour,
                            crate::logic::Cell::Dead => self.cell_dead_colour,
                        }
                    },
                    egui::Stroke::new(1.0, Color32::GRAY),
                );

                layer_painter.add(rect);
            }
        }

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

        // If update is not requested the board will become outdated.
        // This causes higher cpu usage, but only by one/two %.
        ctx.request_repaint();
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
