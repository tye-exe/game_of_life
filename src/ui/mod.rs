#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use egui::Color32;

/// Runs the ui.
pub fn ui_init() -> eframe::Result<()> {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "Game of life",
        native_options,
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
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

    /// The colour of alive cells.
    cell_alive_colour: Color32,
    /// The colour of dead cells.
    cell_dead_colour: Color32,
    /// The size of each cell.
    cell_size: f32,
}

impl Default for MyApp<'_> {
    fn default() -> Self {
        Self {
            label: "Hello world!",
            cell_alive_colour: Color32::WHITE,
            cell_dead_colour: Color32::BLACK,
            cell_size: 5.0,
        }
    }
}

impl MyApp<'static> {
    pub fn new(creation_context: &eframe::CreationContext<'_>) -> Self {
        Default::default()
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
                        .text("Cell size"),
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

            use egui::{pos2, Rect, Rounding, Shape};
            let mut x = 0.0;
            let mut y = 0.0;

            while x < size.x {
                y = 0.0;
                while y < size.y {
                    let rect = Rect::from_two_pos(
                        pos2(x, y),
                        pos2(x + self.cell_size, y + self.cell_size),
                    );
                    let rect_filled = Shape::rect_filled(rect, Rounding::ZERO, {
                        if x % 2.0 == 0.0 {
                            self.cell_alive_colour
                        } else {
                            self.cell_dead_colour
                        }
                    });
                    layer_painter.add(rect_filled);
                    y += self.cell_size;
                }
                x += self.cell_size;
            }
        });
    }
}
