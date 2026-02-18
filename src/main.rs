mod bench_config;
mod executor;
mod helper;
mod monitor_memory;
mod monitor_time;
mod parser;
mod results;
mod warmup;

use crate::{bench_config::BenchConfig, executor::execute_benchmark, parser::Args};

use clap::Parser;
use rayon::ThreadPoolBuilder;
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = Args::parse();

    if !args.output_folder.is_dir() {
        error_print!(
            "{:?} does not exists or is not a directory.",
            args.output_folder
        );
        return ExitCode::FAILURE;
    }

    let bench_config = BenchConfig::try_from(&args.config_file).map_err(|e| {
        error_print!(
            "Could not parse your config file: {}",
            args.config_file.display()
        );
        error_print!("{}", e);
    });

    let Ok(config) = bench_config else {
        return ExitCode::FAILURE;
    };

    println!("{}", config);

    if config.commands.is_empty() {
        error_print!("No commands provided.");
        return ExitCode::FAILURE;
    }

    let thread_pool = ThreadPoolBuilder::new()
        .num_threads(config.threads)
        .build()
        .unwrap();

    let bench_results =
        execute_benchmark(config, thread_pool).expect("Failed to create benchmark results");

    if let Err(e) = bench_results.export_to_csv_files(args.output_folder) {
        error_print!("Could not export the benchmark results: {}", e);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
