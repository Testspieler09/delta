mod bench_config;
mod monitor_memory;
mod parser;

use crate::{bench_config::BenchConfig, monitor_memory::monitor_memory, parser::Args};

use clap::Parser;
use std::{
    process::{Command, ExitCode},
    time::Instant,
};

fn main() -> ExitCode {
    let args = Args::parse();

    let bench_config = BenchConfig::try_from(&args.config_file).map_err(|_| {
        eprintln!(
            "Could not parse your config file: {}",
            args.config_file.display()
        );
    });

    let Ok(config) = bench_config else {
        return ExitCode::FAILURE;
    };

    if config.commands.is_empty() {
        eprintln!("No commands provided.");
        return ExitCode::FAILURE;
    }

    // TODO: add multithreading here
    for cmd_config in config.commands {
        let Ok(mut child) = Command::new(&cmd_config.cmd)
            .spawn()
            .map_err(|_| eprintln!("Could not spawn process with cmd: {:?}", &cmd_config.cmd))
        else {
            return ExitCode::FAILURE;
        };
        let start = Instant::now();
        let Ok((duration, max_mem)) =
            monitor_memory(&mut child).map_err(|_| eprintln!("Failed to read a process memory"))
        else {
            return ExitCode::FAILURE;
        };

        let status = child
            .wait()
            .expect(&format!("Command {:?} was not running", &cmd_config.cmd));
        let duration = start.elapsed();

        // TODO: save the collected data into a csv file or similar
    }

    ExitCode::SUCCESS
}
