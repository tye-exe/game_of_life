use std::path::PathBuf;

use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// The path to the directory which will contain the user configuration data.
    #[arg(short, long, value_name = "DIR")]
    pub(crate) config_path: Option<PathBuf>,
}
