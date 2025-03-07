use std::{error::Error, path::PathBuf, sync::LazyLock};

use app::MyApp;
use app_dirs2::{AppDataType, AppInfo, get_app_dir, get_app_root};
use args::Args;
use clap::Parser;
use gol_lib::{SharedDisplay, Simulator, communication::UiPacket};

mod app;
mod args;
mod file_management;
mod settings;
mod user_actions;

struct Data {
    context: egui::Context,
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let args = Args::parse();

    let mut config_path = args.config_path.unwrap_or(USER_CONFIG_PATH.clone());
    std::fs::create_dir_all(config_path.as_path())?;
    config_path.push("config_data.json");

    let shared_display = SharedDisplay::default();
    let board = gol_simple::Board::new(shared_display.clone());

    let ((ui_sender, ui_receiver), (simulator_sender, simulator_receiver)) =
        gol_lib::create_channels();

    // Start IO thread.
    let io_threads = threadpool::Builder::new()
        .num_threads(1)
        .thread_name("Background IO thread".to_owned())
        .build();

    // Start UI.
    let native_options = eframe::NativeOptions {
        // Takes path to file, not dir.
        persistence_path: Some(config_path),
        ..Default::default()
    };

    // Needs to be initialised, so wrap it in an option.
    let mut simulator = None;

    // The ui has to run on the main thread for compatibility purposes.
    eframe::run_native(
        lang::APP_NAME,
        native_options,
        Box::new(|cc| {
            // Start Simulator in GUI closure to access to GUI context.
            simulator = Some(
                gol_lib::start_simulator_with_callback(
                    board,
                    ui_receiver,
                    simulator_sender,
                    Data {
                        context: cc.egui_ctx.clone(),
                    },
                    |data, is_running| {
                        if *is_running {
                            data.context.request_repaint();
                        }
                    },
                )
                .inspect_err(|_| eprintln!("{}", error_text::CREATE_SIMULATION_THREAD))?,
            );

            Ok(Box::new(MyApp::new(
                cc,
                shared_display,
                ui_sender.clone(),
                simulator_receiver,
                &io_threads,
            )))
        }),
    )
    .inspect_err(|_| eprintln!("{}", error_text::UI_INIT))?;

    // Command similator thread to terminate after the ui is closed.
    if ui_sender.send(UiPacket::Terminate).is_err() {
        log::error!("{}", error_text::COMMAND_SIM_THREAD_TERM)
    };

    // The retuned error does not implement the Error trait so panic instead.
    simulator
        .map(|simulator| simulator.join().ok())
        .flatten()
        .expect(error_text::SIM_THREAD_TERM);

    io_threads.join();

    Ok(())
}

/// The information used to get the default save locations.
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

/// The path to where board saves will be stored.
///
/// On Linux:
/// `/home/<user>/.local/share/game_of_life/saves`
static DEFAULT_SAVE_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    // The only way this can error is if the APP_INFO has empty fields.
    get_app_dir(AppDataType::UserData, &APP_INFO, "saves").unwrap()
});

/// The path to where blueprints will be stored.
///
/// On Linux:
/// `/home/<user>/.local/share/game_of_life/blueprints`
static DEFAULT_BLUEPRINT_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    // The only way this can error is if the APP_INFO has empty fields.
    get_app_dir(AppDataType::UserData, &APP_INFO, "blueprints").unwrap()
});

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
        COMMAND_SIM_THREAD_TERM, "Unable to command similator thread to terminate."
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
        SHARED_DISPLAY_POISIONED, "Unable to read board from simulation."
    }
}
