use std::sync::mpsc;

use logic::{SharedDisplay, Simulator};

mod logic;
mod ui;

fn main() {
    let display: SharedDisplay = Default::default();

    let ((ui_sender, ui_receiver), (simulator_sender, simulator_receiver)) =
        logic::create_channels();

    ui::ui_init(display.clone(), ui_sender, simulator_receiver).unwrap();
    logic::simplistic::Board::new(display.clone(), ui_receiver, simulator_sender);
}
