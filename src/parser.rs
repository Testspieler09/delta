use clap::Parser;
use std::path::PathBuf;

/// Measure and compare commands based on time and memory usage during execution
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(super) struct Args {
    /// The filepath to the config file
    #[arg(short, long, value_name = "FILE")]
    pub config_file: PathBuf,
}
