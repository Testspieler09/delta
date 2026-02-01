use crate::{
    bench_config::BenchConfig,
    info,
    monitor_memory::{measure_memory_usage_over_time, run_and_measure_peak_memory},
    monitor_time::measure_execution_time,
    parser::MeasuringMode,
    results::{BenchResults, CommandResults, ExecutionTimeResult, RunResult},
};

use anyhow::Result;
use rayon::ThreadPool;
use std::{process::ExitCode, time::Duration};

#[derive(Clone)]
pub struct RunConfig {
    pub cmd: String,
    pub args: Vec<String>,
    pub timeout: Duration,
    pub memory_interval: Duration,
}

fn expand_runs(bench_config: &BenchConfig) -> Vec<RunConfig> {
    bench_config
        .commands
        .iter()
        .flat_map(|cmd| {
            let iters = cmd.iterations.unwrap_or(bench_config.iterations);
            (0..iters).map(move |_| RunConfig {
                cmd: cmd.cmd.clone(),
                args: cmd.args.clone(),
                timeout: cmd
                    .max_execution_time
                    .unwrap_or(bench_config.max_execution_time),
                memory_interval: cmd
                    .memory_sampling_interval
                    .unwrap_or(bench_config.memory_sampling_interval),
            })
        })
        .collect()
}

pub(super) fn execute_benchmark(
    bench_config: BenchConfig,
    thread_pool: ThreadPool,
    measure_mem_once: bool,
    memory_measuring_mode: MeasuringMode,
) -> Result<BenchResults, ExitCode> {
    let runs = expand_runs(&bench_config);

    let mut command_results = bench_config
        .commands
        .iter()
        .map(|cmd| CommandResults {
            cmd: cmd.cmd.clone(),
            args: cmd.args.clone(),
            runs: Vec::with_capacity(2),
        })
        .collect::<Vec<CommandResults>>();
    let cmd_results_size = command_results.len();

    let times = measure_execution_time(&runs, &thread_pool).map_err(|_| ExitCode::FAILURE)?;

    for (i, _run) in runs.iter().enumerate() {
        let exec_time_result = ExecutionTimeResult {
            duration: times[i].unwrap_or_default(),
        };

        let command_result = &mut command_results[i % cmd_results_size];
        command_result.runs.push(RunResult::Time(exec_time_result));
    }

    info!("Starting to measure memory usage");
    // TODO: adjust this with measure_mem_once in mind
    let memory_stats: Vec<RunResult> =
        match (memory_measuring_mode, measure_mem_once) {
            (MeasuringMode::Timeline, true) => measure_memory_usage_over_time(&runs, &thread_pool)
                .map_err(|_| ExitCode::FAILURE)?,
            (MeasuringMode::Timeline, false) => measure_memory_usage_over_time(&runs, &thread_pool)
                .map_err(|_| ExitCode::FAILURE)?,
            (MeasuringMode::Maximum, true) => run_and_measure_peak_memory(&runs),
            (MeasuringMode::Maximum, false) => run_and_measure_peak_memory(&runs),
        };
    info!("Benchmarking completed. Preparing results for output.");

    Ok(BenchResults {
        commands: command_results,
    })
}
