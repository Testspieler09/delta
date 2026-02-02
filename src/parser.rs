use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(super) struct Args {
    /// The filepath to the config file
    #[arg(short = 'f', long, value_name = "FILE")]
    pub config_file: PathBuf,

    /// The filepath to the output folder
    #[arg(short = 'o', long, value_name = "FOLDER")]
    pub output_folder: PathBuf,
}
