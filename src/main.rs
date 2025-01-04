use std::{error::Error, path::PathBuf, sync::LazyLock, thread, time::Duration};

use app_dirs2::{get_app_dir, get_app_root, AppDataType, AppInfo};
use logic::{SharedDisplay, Simulator, SimulatorPacket};

mod logic;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    init_directories().inspect_err(|_| eprintln!("{}", error_text::DIRECTORY_CREATION))?;

    let display: SharedDisplay = Default::default();

    let ((ui_sender, ui_receiver), (simulator_sender, simulator_receiver)) =
        logic::create_channels();

    // Creates a separate thread for the simulation to run in.
    let simulator_display = display.clone();

    let simulator_thread = thread::Builder::new()
        .name("Simulator_Thread".into())
        .spawn(move || {
            let send_packet = |packet: SimulatorPacket| match simulator_sender.send(packet) {
                Ok(_) => {}
                Err(_) => {
                    std::panic!("{}", error_text::UI_CLOSED_COMS)
                }
            };

            let mut board = logic::simplistic::Board::new(simulator_display);
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
                            std::panic!("{}", error_text::UI_CLOSED_COMS);
                        }
                    };

                    match ui_packet {
                        logic::UiPacket::DisplayArea { new_area } => {
                            board.set_display_area(new_area);
                            display_needs_updating = true;
                        }
                        logic::UiPacket::Set {
                            position,
                            cell_state,
                        } => {
                            board.set(position, cell_state);
                            display_needs_updating = true;
                        }
                        logic::UiPacket::SaveBoard => {
                            let board = board.save_board();
                            send_packet(SimulatorPacket::BoardSave { board });
                        }
                        logic::UiPacket::LoadBoard { board: new_board } => {
                            board.load_board(new_board);
                            display_needs_updating = true;
                        }
                        logic::UiPacket::SaveBlueprint { area } => {
                            let blueprint = board.save_blueprint(area);
                            send_packet(SimulatorPacket::BlueprintSave { blueprint });
                        }
                        logic::UiPacket::LoadBlueprint {
                            load_position,
                            blueprint,
                        } => {
                            board.load_blueprint(load_position, blueprint);
                            display_needs_updating = true;
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
                        logic::UiPacket::Terminate => return (),
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
        });

    let simulator_thread =
        simulator_thread.inspect_err(|_| eprintln!("{}", error_text::CREATE_SIMULATION_THREAD))?;

    // The ui has to run on the main thread for compatibility purposes.
    ui::ui_init(display, ui_sender, simulator_receiver)
        .inspect_err(|_| eprintln!("{}", error_text::UI_INIT))?;

    // The retuned error does not implement the Error trait so panic instead.
    simulator_thread.join().expect(error_text::SIM_THREAD_TERM);

    Ok(())
}

pub const APP_INFO: AppInfo = AppInfo {
    name: "game_of_life-tye",
    author: "tye",
};

/// The path to where user configuration will be stored.
/// This path is guaranteed to exist.
///
/// On Linux:
/// `/home/<user>/.config/game_of_life`
static USER_CONFIG_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    // The only way this can error is if the APP_INFO has empty fields.
    get_app_root(AppDataType::UserConfig, &APP_INFO).unwrap()
});

/// The path to where user data will be stored.
/// This path is guaranteed to exist.
///
/// On Linux:
/// `/home/<user>/.local/share/game_of_life`
static USER_DATA_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    // The only way this can error is if the APP_INFO has empty fields.
    get_app_root(AppDataType::UserData, &APP_INFO).unwrap()
});

/// The path to where board saves will be stored.
/// This path is guaranteed to exist.
///
/// On Linux:
/// `/home/<user>/.local/share/game_of_life/saves`
static USER_SAVE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    // The only way this can error is if the APP_INFO has empty fields.
    get_app_dir(AppDataType::UserData, &APP_INFO, "saves").unwrap()
});

/// The path to where blueprints will be stored.
/// This path is guaranteed to exist.
///
/// On Linux:
/// `/home/<user>/.local/share/game_of_life/blueprints`
static USER_BLUEPRINT_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    // The only way this can error is if the APP_INFO has empty fields.
    get_app_dir(AppDataType::UserData, &APP_INFO, "blueprints").unwrap()
});

/// Creates the directories used by this application.
fn init_directories() -> Result<(), std::io::Error> {
    std::fs::create_dir_all(USER_CONFIG_PATH.as_path())?;
    std::fs::create_dir_all(USER_DATA_PATH.as_path())?;
    std::fs::create_dir_all(USER_SAVE_PATH.as_path())?;
    std::fs::create_dir_all(USER_BLUEPRINT_PATH.as_path())?;
    Ok(())
}

/// Creates a public constant string with the name as the name of the constant
/// and the text as the value of the string.
///
/// # Examples
/// ```
/// lang!{QUOTE, "Ya like jazz?"}
/// assert_eq!(QUOTE, "Ya like jazz?");
/// ```
#[macro_export]
macro_rules! lang {
    {$($name:tt, $text:literal);*} => {
        $(
        pub const $name: &str = $text;
        )*
    };
}

pub mod error_text {
    lang! {
        UI_CLOSED_COMS, "UI closed communication to simulation!";
        CREATE_SIMULATION_THREAD, "Unable to create thread for board simulation at OS level.";
        UI_INIT, "Unable to initialis UI graphical context.";
        SIM_THREAD_TERM, "Simulator thread was unable to gracefully terminate";
        COMMAND_SIM_THREAD_TERM, "Unable to command similator thread to terminate.";
        DIRECTORY_CREATION, "Unable to created required directory for this program to run."
    }
}
