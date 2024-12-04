use std::{thread, time::Duration};

use logic::{SharedDisplay, Simulator, SimulatorPacket};

mod logic;
mod ui;

fn main() {
    let display: SharedDisplay = Default::default();

    let ((ui_sender, ui_receiver), (simulator_sender, simulator_receiver)) =
        logic::create_channels();

    // Creates a separate thread for the simulation to run in.
    let simulator_display = display.clone();
    let simulator_thread = std::thread::spawn(move || {
        let send_packet = |packet: SimulatorPacket| match simulator_sender.send(packet) {
            Ok(_) => {}
            Err(_) => {
                std::panic!("UI closed communication!")
            }
        };

        let mut board = logic::simplistic::Board::new(simulator_display);
        // Used to control the ticks per second.
        let mut tick_rate_limiter = spin_sleep_util::interval(Duration::from_secs(1));
        tick_rate_limiter.set_missed_tick_behavior(spin_sleep_util::MissedTickBehavior::Skip);

        let mut is_running = false;
        let mut run_until = None;
        let mut tick_rate_limited = false;

        loop {
            // Process all received packets.
            loop {
                use std::sync::mpsc::TryRecvError;
                let ui_packet = match ui_receiver.try_recv() {
                    Ok(ui_packet) => ui_packet,
                    Err(TryRecvError::Empty) => {
                        break;
                    }
                    Err(TryRecvError::Disconnected) => {
                        std::panic!("UI closed communication!");
                    }
                };

                match ui_packet {
                    logic::UiPacket::DisplayArea { new_area } => board.set_display_area(new_area),
                    logic::UiPacket::Set {
                        position,
                        cell_state,
                    } => board.set(position, cell_state),
                    logic::UiPacket::SaveBoard => {
                        let board = board.save_board();
                        send_packet(SimulatorPacket::BoardSave { board });
                    }
                    logic::UiPacket::LoadBoard { board: new_board } => {
                        let status = board.load_board(new_board);
                        send_packet(SimulatorPacket::BoardLoadResult { status });
                    }
                    logic::UiPacket::SaveBlueprint { area } => {
                        let blueprint = board.save_blueprint(area);
                        send_packet(SimulatorPacket::BlueprintSave { blueprint });
                    }
                    logic::UiPacket::LoadBlueprint {
                        load_position,
                        blueprint,
                    } => {
                        let status = board.load_blueprint(load_position, blueprint);
                        send_packet(SimulatorPacket::BlueprintLoadResult { status })
                    }
                    logic::UiPacket::Start => is_running = true,
                    logic::UiPacket::StartUntil { generation } => {
                        is_running = true;
                        run_until = Some(generation);
                    }
                    logic::UiPacket::Stop => is_running = false,
                    logic::UiPacket::SimulationSpeed { speed } => match speed.get() {
                        Some(ticks_per_second) => {
                            tick_rate_limiter
                                .set_period(Duration::from_secs(1) / ticks_per_second.get());
                            tick_rate_limited = true;
                        }
                        None => {
                            tick_rate_limited = false;
                        }
                    },
                }
            }

            // If the game is not running then wait for ≈ 100ms before performing any updates to save resources.
            if !is_running {
                thread::sleep(Duration::from_millis(100));
                continue;
            }

            if let Some(generation) = run_until {
                if generation >= board.get_generation() {
                    is_running = false;
                    continue;
                }
            }

            if tick_rate_limited {
                tick_rate_limiter.tick();
            }

            board.tick();
            board.update_display();
        }
    });

    // The ui has to run on the main thread for compatibility purposes.
    ui::ui_init(display, ui_sender, simulator_receiver).unwrap();

    simulator_thread.join();
}
