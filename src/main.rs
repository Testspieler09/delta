mod bench_config;
mod executor;
mod monitor_memory;
mod parser;

use crate::{bench_config::BenchConfig, executor::execute_benchmark, parser::Args};

use clap::Parser;
use rayon::ThreadPoolBuilder;
use std::process::ExitCode;

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

    let thread_pool = ThreadPoolBuilder::new()
        .num_threads(config.threads)
        .build()
        .unwrap();

    let _ = execute_benchmark(config, thread_pool);

    ExitCode::SUCCESS
}
