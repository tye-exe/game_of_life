use std::sync::mpsc;

use logic::{SharedDisplay, Simulator};

mod logic;
mod ui;

fn main() {
    let display: SharedDisplay = Default::default();

    let ((ui_sender, ui_receiver), (simulator_sender, simulator_receiver)) =
        logic::create_channels();

    // Creates a separate thread for the simulation to run in.
    let simulator_display = display.clone();
    let simulator_thread = std::thread::spawn(move || {
        let mut board =
            logic::simplistic::Board::new(simulator_display, ui_receiver, simulator_sender);
        for _ in 0..100 {
            board.tick();
            board.update_display();
            board.ui_communication();
        }
    });

    // The ui has to run on the main thread for compatibility purposes.
    ui::ui_init(display, ui_sender, simulator_receiver).unwrap();

    simulator_thread.join();
}
