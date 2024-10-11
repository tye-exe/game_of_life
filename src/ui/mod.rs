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
            cell_size: 1.0,
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
        egui::CentralPanel::default().show(ctx, |ui| {
            let layer_painter = board_painter(ctx);

            use egui::{pos2, Rect, Rounding, Shape};
            for x in 0..100 {
                for y in 0..100 {
                    let x = x as f32;
                    let y = y as f32;
                    let rect = Rect::from_two_pos(pos2(x, y), pos2(x + 1.0, y + 1.0));
                    let rect_filled = Shape::rect_filled(rect, Rounding::ZERO, {
                        if x % 2.0 == 0.0 {
                            self.cell_alive_colour
                        } else {
                            self.cell_dead_colour
                        }
                    });
                    // egui::Rect::from_pos()
                    layer_painter.add(rect_filled);
                }
            }
        });

        egui::SidePanel::right("Right_Panel").show(ctx, |ui| {
            let pointer_latest_pos = ctx.pointer_latest_pos();
            if let Some(pos) = pointer_latest_pos {
                ui.heading(pos.to_string());
            }
        });
    }
}
