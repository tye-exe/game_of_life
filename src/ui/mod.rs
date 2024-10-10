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

/// The struct that contains the data for the gui of my app.
struct MyApp<'a> {
    label: &'a str,

    /// The colour of alive cells.
    cell_alive_colour: Color32,
    /// The colour of dead cells.
    cell_dead_colour: Color32,
}

impl Default for MyApp<'_> {
    fn default() -> Self {
        Self {
            label: "Hello world!",
            cell_alive_colour: Color32::WHITE,
            cell_dead_colour: Color32::BLACK,
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
            ui.heading(self.label);
            let layer_id = egui::LayerId::new(egui::Order::Background, BOARD_ID.into());
            let layer_painter = ctx.layer_painter(layer_id);
            let rect = egui::Rect::from_min_max(egui::pos2(1.0, 1.0), egui::pos2(100.0, 100.0));
            let rect_filled =
                egui::Shape::rect_filled(rect, egui::Rounding::ZERO, self.cell_dead_colour);
            layer_painter.add(rect_filled);
        });

        egui::SidePanel::right("Right_Panel").show(ctx, |ui| {
            let pointer_latest_pos = ctx.pointer_latest_pos();
            if let Some(pos) = pointer_latest_pos {
                ui.heading(pos.to_string());
            }
        });
    }
}
