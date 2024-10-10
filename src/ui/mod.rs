#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

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

const BOARD_ID: &str = "board";

struct MyApp<'a> {
    label: &'a str,
}

impl Default for MyApp<'_> {
    fn default() -> Self {
        Self {
            label: "Hello world!",
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
            let colour = egui::Color32::from_rgb(0, 0, 0);
            let rect_filled = egui::Shape::rect_filled(rect, egui::Rounding::ZERO, colour);
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
