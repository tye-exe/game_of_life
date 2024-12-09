#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui::{pos2, Color32, Painter, Rect};

use crate::{
    error_text,
    logic::{BoardDisplay, SharedDisplay, SimulatorReceiver, UiSender},
};

mod lang {
    use crate::lang;

    lang! {
        APP_NAME, "Game Of Life";
        CELL_SIZE_SLIDER, "Cell Size"
    }
}

/// Runs the ui.
pub fn ui_init(
    display: SharedDisplay,
    ui_sender: UiSender,
    simulator_receiver: SimulatorReceiver,
) -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
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
    if let Err(_) = ui_sender.send(crate::logic::UiPacket::Terminate) {
        eprintln!("{}", error_text::COMMAND_SIM_THREAD_TERM)
    };
    run_native
}

/// The egui id for the board where the cells are being displayed.
const BOARD_ID: &str = "board";

/// The struct that contains the data for the gui of my app.
struct MyApp<'a> {
    label: &'a str,

    /// The updated display produced by the simulator.
    display_update: SharedDisplay,
    /// The current display being rendered.
    display_cache: BoardDisplay,

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
        creation_context: &eframe::CreationContext<'_>,
        display: SharedDisplay,
        ui_sender: UiSender,
        simulator_receiver: SimulatorReceiver,
    ) -> Self {
        MyApp {
            label: "Hello world",
            display_update: display,
            display_cache: Default::default(),
            cell_alive_colour: Color32::WHITE,
            cell_dead_colour: Color32::BLACK,
            cell_size: 5.0,
            ui_sender,
            simulator_receiver,
        }
    }
}

impl eframe::App for MyApp<'_> {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Stores the size the board will take up.
        let mut board_rect = Rect::from_min_max(
            (0.0, 0.0).into(),
            ctx.input(|i| i.screen_rect()).right_bottom(),
        );

        let top_size = egui::TopBottomPanel::top("Top_Panel")
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.button("Start");
                    ui.button("Stop");
                });
            })
            .response
            .rect
            .size();

        // Account for top panel.
        *board_rect.top_mut() += top_size.y;
        *board_rect.bottom_mut() += top_size.y;

        // Draws the right side panel & gets the size of it.
        let panel_size = egui::SidePanel::right("Right_Panel")
            .show(ctx, |ui| {
                let pointer_latest_pos = ctx.pointer_latest_pos();
                if let Some(pos) = pointer_latest_pos {
                    ui.heading(pos.to_string());
                    ui.heading(board_rect.size().to_string());
                }

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

        // Draws the board panel last, so that the available size to draw is known
        egui::CentralPanel::default().show(ctx, |ui| {
            // Creates the painter once & reuses it
            let layer_painter = Painter::new(
                ctx.clone(), // ctx is cloned in egui implementations.
                egui::LayerId::new(egui::Order::Background, BOARD_ID.into()),
                board_rect,
            );

            use egui::{pos2, Rect, Rounding, Stroke};

            let get_x = self.display_cache.get_x();
            let get_y = self.display_cache.get_y();

            let cell_x = board_rect.x_range().span() / get_x.get() as f32;
            let cell_y = board_rect.y_range().span() / get_y.get() as f32;

            for x in 0..get_x.get() {
                let x_pos = x as f32 * cell_x;

                for y in 0..get_y.get() {
                    let y_pos = y as f32 * cell_y;
                    let rect = Rect::from_two_pos(
                        pos2(x_pos, y_pos),
                        pos2(x_pos + cell_x, y_pos + cell_y),
                    );

                    let rect = egui::epaint::RectShape::new(
                        rect,
                        Rounding::ZERO,
                        {
                            match self.display_cache.get_cell((x as i32, y as i32)) {
                                crate::logic::Cell::Alive => self.cell_alive_colour,
                                crate::logic::Cell::Dead => self.cell_dead_colour,
                            }
                        },
                        Stroke::new(1.0, Color32::GRAY),
                    );

                    layer_painter.add(rect);
                }
            }
        });
    }
}
