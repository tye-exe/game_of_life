mod area;
pub mod board_data;
mod cell;
pub mod communication;
mod display;
mod position;
mod simulator;

pub use area::Area;
pub use cell::Cell;
pub use display::BoardDisplay;
pub use position::GlobalPosition;
pub use simulator::Simulator;

use communication::{SimulatorPacket, UiPacket};
use std::sync::{mpsc, Arc, Mutex};
use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration,
};

/// A pointer to the [`Mutex`] used to share the display board.
/// The time either the ui or the [`Simulator`] will hold a lock on the [`Mutex`] is not guaranteed.
pub type SharedDisplay = Arc<Mutex<Option<BoardDisplay>>>;

/// The [`Receiver`] for [`UiPacket`]s from the ui.
///
/// [`Receiver`]: std::sync::mpsc::Receiver
pub type UiReceiver = mpsc::Receiver<UiPacket>;
/// The [`Sender`] for [`UiPacket`]s being sent from the ui.
/// Only the ui should ever have this [`Sender`].
///
/// [`Sender`]: std::sync::mpsc::Sender
pub type UiSender = mpsc::Sender<UiPacket>;
/// The [`Receiver`] for [`SimulatorPacket`]s from the [`Simulator`].
///
/// [`Receiver`]: std::sync::mpsc::Receiver
pub type SimulatorReceiver = mpsc::Receiver<SimulatorPacket>;
/// The [`Sender`] for [`SimulatorPacket`]s being sent from the [`Simulator`].
/// Only the [`Simulator`] should ever have this [`Sender`].
///
/// [`Sender`]: std::sync::mpsc::Sender
pub type SimulatorSender = mpsc::Sender<SimulatorPacket>;

/// Creates the channels for communication between the [`Simulator`] & the UI.
pub fn create_channels() -> ((UiSender, UiReceiver), (SimulatorSender, SimulatorReceiver)) {
    (mpsc::channel(), mpsc::channel())
}

pub fn start_simulator(
    mut board: impl Simulator + 'static,
    ui_receiver: Receiver<UiPacket>,
    simulator_sender: Sender<SimulatorPacket>,
) -> Result<thread::JoinHandle<()>, std::io::Error> {
    thread::Builder::new()
        .name("Simulator_Thread".into())
        .spawn(move || {
            let send_packet = |packet: SimulatorPacket| match simulator_sender.send(packet) {
                Ok(_) => {}
                Err(_) => {
                    std::panic!("{}", UI_CLOSED_COMS)
                }
            };

            // Used to control the ticks per second.
            let mut tick_rate_limiter = spin_sleep_util::interval(Duration::from_secs(1));
            tick_rate_limiter.set_missed_tick_behavior(spin_sleep_util::MissedTickBehavior::Skip);

            let mut is_running = false;
            let mut run_until = None;
            let mut tick_rate_limited = false;
            let mut display_needs_updating = false;

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
                            std::panic!("{}", UI_CLOSED_COMS);
                        }
                    };

                    match ui_packet {
                        UiPacket::DisplayArea { new_area } => {
                            board.set_display_area(new_area);
                            display_needs_updating = true;
                        }
                        UiPacket::Set {
                            position,
                            cell_state,
                        } => {
                            board.set(position, cell_state);
                            display_needs_updating = true;
                        }
                        UiPacket::SaveBoard => {
                            let board = board.save_board();
                            send_packet(SimulatorPacket::BoardSave { board });
                        }
                        UiPacket::LoadBoard { board: new_board } => {
                            board.load_board(new_board);
                            display_needs_updating = true;
                        }
                        UiPacket::SaveBlueprint { area } => {
                            let blueprint = board.save_blueprint(area);
                            send_packet(SimulatorPacket::BlueprintSave { blueprint });
                        }
                        UiPacket::LoadBlueprint {
                            load_position,
                            blueprint,
                        } => {
                            board.load_blueprint(load_position, blueprint);
                            display_needs_updating = true;
                        }
                        UiPacket::Start => is_running = true,
                        UiPacket::StartUntil { generation } => {
                            is_running = true;
                            run_until = Some(generation);
                        }
                        UiPacket::Stop => is_running = false,
                        UiPacket::SimulationSpeed { speed } => match speed.get() {
                            Some(ticks_per_second) => {
                                tick_rate_limiter
                                    .set_period(Duration::from_secs(1) / ticks_per_second.get());
                                tick_rate_limited = true;
                            }
                            None => {
                                tick_rate_limited = false;
                            }
                        },
                        UiPacket::Terminate => return,
                    }
                }

                // If the game is not running then wait for ≈ 100ms before performing any updates to save resources.
                if !is_running {
                    if display_needs_updating {
                        board.update_display();
                        display_needs_updating = !display_needs_updating;
                    }

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
        })
}

const UI_CLOSED_COMS: &str = "UI closed communication to simulation!";
