use std::{error::Error, path::PathBuf, sync::LazyLock, thread, time::Duration};

use app::MyApp;
use app_dirs2::{get_app_dir, get_app_root, AppDataType, AppInfo};
use gol_lib::{communication::UiPacket, SharedDisplay, Simulator};

mod app;
mod file_management;
mod settings;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    init_directories().inspect_err(|_| eprintln!("{}", error_text::DIRECTORY_CREATION))?;

    let shared_display = SharedDisplay::default();
    let board = gol_simple::Board::new(shared_display.clone());

    let ((ui_sender, ui_receiver), (simulator_sender, simulator_receiver)) =
        gol_lib::create_channels();

    // Start Simulator.
    let simulator = gol_lib::start_simulator(board, ui_receiver, simulator_sender)
        .inspect_err(|_| eprintln!("{}", error_text::CREATE_SIMULATION_THREAD))?;

    // Start UI.
    let native_options = eframe::NativeOptions {
        ..Default::default()
    };

    // The ui has to run on the main thread for compatibility purposes.
    eframe::run_native(
        lang::APP_NAME,
        native_options,
        Box::new(|cc| {
            Ok(Box::new(MyApp::new(
                cc,
                shared_display,
                ui_sender.clone(),
                simulator_receiver,
            )))
        }),
    )
    .inspect_err(|_| eprintln!("{}", error_text::UI_INIT))?;

    // Command similator thread to terminate after the ui is closed.
    if ui_sender.send(UiPacket::Terminate).is_err() {
        log::error!("{}", error_text::COMMAND_SIM_THREAD_TERM)
    };

    // The retuned error does not implement the Error trait so panic instead.
    simulator.join().expect(error_text::SIM_THREAD_TERM);

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

mod error_text {
    lang! {
        CREATE_SIMULATION_THREAD, "Unable to create thread for board simulation at OS level.";
        UI_INIT, "Unable to initialise UI graphical context.";
        SIM_THREAD_TERM, "Simulator thread was unable to gracefully terminate";
        COMMAND_SIM_THREAD_TERM, "Unable to command similator thread to terminate.";
        DIRECTORY_CREATION, "Unable to created required directory for this program to run."
    }
}

mod lang {
    use crate::lang;

    lang! {
        APP_NAME, "Game Of Life";
        UNRECOVERABLE_ERROR_HEADER, "Encountered Unrecoverable Error";
        ERROR_MESSAGE, "Error: ";
        ERROR_ADVICE, "Please restart the application.";
        SEND_ERROR, "Unable to send packet to simulation.";
        RECEIVE_ERROR, "Unable to receive data from simulation.";
        SHARED_DISPLAY_POISIONED, "Unable to read board from simulation.";
        SETTINGS_CLOSE, "Close";
        SETTINGS_RESET, "Reset";
        SETTINGS_LABEL, "Settings";
        SETTINGS_CELL_HEADER, "Cells";
        SETTINGS_KEYBIND_HEADER, "Keybinds";
        SETTINGS_CELL_ALIVE_COLOUR, "Cell alive colour:";
        SETTINGS_CELL_DEAD_COLOUR, "Cell dead colour:";
        SETTINGS_CELL_SIZE, "Cell size:";
        SETTINGS_KEYBIND_SIMULATION_TOGGLE, "Toggle Simulation:";
        SETTINGS_KEYBIND_SETTINGS_MENU_TOGGLE, "Toggle Settings Menu:"
    }
}
