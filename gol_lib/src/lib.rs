mod area;
mod cell;
pub mod communication;
mod display;
pub mod persistence;
mod position;
mod simulator;

pub use area::Area;
pub use cell::Cell;
pub use display::BoardDisplay;
pub use position::GlobalPosition;
pub use simulator::Simulator;

use communication::{SimulatorPacket, UiPacket};
use std::marker::Send;
use std::sync::{Arc, Mutex, mpsc};
use std::{
    sync::mpsc::{Receiver, Sender},
    thread,
    time::Duration,
};

const UI_CLOSED_COMS: &str = "UI closed communication to simulation!";

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

/// Starts the simulation on a new thread without a callback.
///
/// For more information see [`start_simulator_with_callback`].
pub fn start_simulator(
    simulator: impl Simulator + 'static,
    ui_receiver: Receiver<UiPacket>,
    simulator_sender: Sender<SimulatorPacket>,
) -> Result<thread::JoinHandle<()>, std::io::Error> {
    start_simulator_with_callback(simulator, ui_receiver, simulator_sender, (), |_, _| {})
}

/// Starts the given simulation on a new thread.
/// The given callback is called on every tick of the simulation.
/// Due to this the callback **should not** be computationally intensive.
///
/// The callback will not have any effect on the state of the simulation.
/// The only value it can mutate is the given `Data` value, which allows the callback to persist its own state between simulation ticks.
///
/// # Panics
/// If the callback panics, this will be propagrated to the simulation thread and terminate the simulation.
pub fn start_simulator_with_callback<Data, Callback>(
    mut simulator: impl Simulator + 'static,
    ui_receiver: Receiver<UiPacket>,
    simulator_sender: Sender<SimulatorPacket>,
    mut data: Data,
    mut callback: Callback,
) -> Result<thread::JoinHandle<()>, std::io::Error>
where
    Callback: FnMut(&mut Data, IsRunning) + Send + 'static,
    Data: Send + 'static,
{
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
                            simulator.set_display_area(new_area);
                            display_needs_updating = true;
                        }
                        UiPacket::Set {
                            position,
                            cell_state,
                        } => {
                            simulator.set(position, cell_state);
                            display_needs_updating = true;
                        }
                        UiPacket::SaveBoard => {
                            let board = simulator.save_board();
                            send_packet(SimulatorPacket::BoardSave { board });
                        }
                        UiPacket::LoadBoard { board: new_board } => {
                            simulator.load_board(new_board);
                            display_needs_updating = true;
                        }
                        UiPacket::SaveBlueprint { area } => {
                            let blueprint = simulator.save_blueprint(area);
                            send_packet(SimulatorPacket::BlueprintSave { blueprint });
                        }
                        UiPacket::LoadBlueprint {
                            load_position,
                            blueprint,
                        } => {
                            simulator.load_blueprint(load_position, blueprint);
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

                // Execute the user defined callback with the user data.
                callback(&mut data, IsRunning(is_running));

                // If the game is not running then wait for ≈ 100ms before performing any updates to save resources.
                if !is_running {
                    if display_needs_updating {
                        simulator.update_display();
                        display_needs_updating = !display_needs_updating;
                    }

                    thread::sleep(Duration::from_millis(100));
                    continue;
                }

                if let Some(generation) = run_until {
                    if generation >= simulator.get_generation() {
                        is_running = false;
                        continue;
                    }
                }

                if tick_rate_limited {
                    tick_rate_limiter.tick();
                }

                simulator.tick();
                simulator.update_display();
            }
        })
}

/// This boolean value represents whether the simulation is currently running or not.
///
/// If it is true, the simulation is running.
/// If it is false, the simulation is not running.
///
/// This value **cannot** be changed to control the state of the simulation.
pub struct IsRunning(bool);

impl std::ops::Deref for IsRunning {
    type Target = bool;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
