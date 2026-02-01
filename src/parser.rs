use clap::{ArgAction, Parser, ValueEnum};
use std::path::PathBuf;

#[derive(Clone, Copy, Debug, PartialEq, Eq, ValueEnum)]
pub(super) enum MeasuringMode {
    Timeline,
    Maximum,
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(super) struct Args {
    /// The filepath to the config file
    #[arg(short = 'f', long, value_name = "FILE")]
    pub config_file: PathBuf,

    /// The filepath to the output folder
    #[arg(short = 'o', long, value_name = "FOLDER")]
    pub output_folder: PathBuf,

    /// Only measure the memory usage on first execution
    #[arg(short='x', long, action = ArgAction::SetTrue)]
    pub measure_mem_once: bool,

    /// Mode for the memory measurment
    #[arg(
        short='m',
        long,
        value_enum,
        default_value_t = MeasuringMode::Timeline
    )]
    pub memory_measuring_mode: MeasuringMode,
}
