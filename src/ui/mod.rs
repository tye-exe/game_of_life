#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui::Color32;

use crate::logic::{BoardDisplay, SharedDisplay, SimulatorReceiver, UiSender};

mod lang {
    /// Creates a public constant string with the name as the name of the constant
    /// and the text as the value of the string.
    ///
    /// # Examples
    /// ```
    /// lang!(QUOTE, "Ya like jazz?");
    /// assert_eq!(QUOTE, "Ya like jazz?");
    /// ```
    macro_rules! lang {
        ($name:tt, $text:literal) => {
            pub const $name: &str = $text;
        };
    }

    lang!(APP_NAME, "Game Of Life");
    lang!(CELL_SIZE_SLIDER, "Cell Size");
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
        eprintln!("Unable to command similator thread to terminate.")
    };
    run_native
}

/// The egui id for the board where the cells are being displayed.
const BOARD_ID: &str = "board";

/// Creates the painter that will display the board for the given context.
fn board_painter(ctx: &egui::Context) -> egui::Painter {
    let layer_id = egui::LayerId::new(egui::Order::Background, BOARD_ID.into());
    ctx.layer_painter(layer_id)
}

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
        let mut size = ctx.input(|i| i.screen_rect()).size();

        // Draws the right side panel & gets the size of it.
        let panel_size = egui::SidePanel::right("Right_Panel")
            .show(ctx, |ui| {
                let pointer_latest_pos = ctx.pointer_latest_pos();
                if let Some(pos) = pointer_latest_pos {
                    ui.heading(pos.to_string());
                    ui.heading(size.to_string());
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
        size.x -= panel_size.x;

        // Draws the board panel last, so that the available size to draw is known
        egui::CentralPanel::default().show(ctx, |ui| {
            // Creates the painter once & reuses it
            let layer_painter = board_painter(ctx);

            use egui::{pos2, Rect, Rounding, Shape, Stroke};
            // let mut x = 0.0;
            // let mut y = 0.0;

            let get_x = self.display_cache.get_x();
            let get_y = self.display_cache.get_y();

            let cell_x = size.x / get_x.get() as f32;
            let cell_y = size.y / get_y.get() as f32;

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
