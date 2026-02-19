use crate::{
    bench_config::{BenchConfig, MeasuringMode},
    info,
    monitor_memory::measure_memory_for_runs,
    monitor_time::measure_execution_time,
    results::{BenchResults, CommandResults, ExecutionTimeResult, RunResult},
};

use anyhow::{Context, Result};
use rayon::ThreadPool;
use std::{path::PathBuf, process::ExitCode, time::Duration};

#[derive(Clone)]
pub struct RunConfig {
    pub cmd_index: usize,
    pub cmd: String,
    pub args: Vec<String>,
    pub timeout: Duration,
    pub memory_interval: Duration,
    pub memory_measuring_mode: MeasuringMode,
}

fn expand_runs_for_time_measurment(bench_config: &BenchConfig) -> Vec<RunConfig> {
    bench_config
        .commands
        .iter()
        .enumerate()
        .flat_map(|(cmd_index, cmd)| {
            let iters = cmd.iterations.unwrap_or(bench_config.iterations);
            (0..iters).map(move |_| RunConfig {
                cmd_index,
                cmd: cmd.cmd.clone(),
                args: cmd.args.clone(),
                timeout: cmd
                    .max_execution_time
                    .unwrap_or(bench_config.max_execution_time),
                memory_interval: cmd
                    .memory_sampling_interval
                    .unwrap_or(bench_config.memory_sampling_interval),
                memory_measuring_mode: cmd
                    .memory_measuring_mode
                    .unwrap_or(bench_config.memory_measuring_mode),
            })
        })
        .collect()
}

fn expand_runs_for_memory_measurment(bench_config: &BenchConfig) -> Vec<RunConfig> {
    bench_config
        .commands
        .iter()
        .enumerate()
        .flat_map(|(cmd_index, cmd)| {
            let iters = if cmd
                .measure_mem_once
                .unwrap_or(bench_config.measure_mem_once)
            {
                1
            } else {
                cmd.iterations.unwrap_or(bench_config.iterations)
            };

            (0..iters).map(move |_| RunConfig {
                cmd_index,
                cmd: cmd.cmd.clone(),
                args: cmd.args.clone(),
                timeout: cmd
                    .max_execution_time
                    .unwrap_or(bench_config.max_execution_time),
                memory_interval: cmd
                    .memory_sampling_interval
                    .unwrap_or(bench_config.memory_sampling_interval),
                memory_measuring_mode: cmd
                    .memory_measuring_mode
                    .unwrap_or(bench_config.memory_measuring_mode),
            })
        })
        .collect()
}

pub(super) fn execute_benchmark_streaming(
    bench_config: BenchConfig,
    thread_pool: ThreadPool,
    output_folder: PathBuf,
) -> Result<(), ExitCode> {
    let timing_runs = expand_runs_for_time_measurment(&bench_config);
    let mem_runs = expand_runs_for_memory_measurment(&bench_config);

    let time_dir = output_folder.join("time");
    let mem_dir = output_folder.join("memory");

    std::fs::create_dir_all(&time_dir).map_err(|_| ExitCode::FAILURE)?;
    std::fs::create_dir_all(&mem_dir).map_err(|_| ExitCode::FAILURE)?;

    let mut command_results: Vec<CommandResults> = bench_config
        .commands
        .iter()
        .map(|cmd| CommandResults {
            cmd: cmd.cmd.clone(),
            args: cmd.args.clone(),
            runs: Vec::with_capacity(cmd.iterations.unwrap_or(bench_config.iterations)),
        })
        .collect();

    let _ = measure_execution_time(
        &timing_runs,
        (&bench_config).into(),
        Some(|run_idx, run_cfg: &RunConfig, elapsed: Option<Duration>| {
            let cmd_idx = run_cfg.cmd_index;
            let run_result = RunResult::Time(ExecutionTimeResult {
                duration: elapsed.unwrap_or_default(),
            });

            // Save to command results
            command_results[cmd_idx].runs.push(run_result.clone());

            // Export immediately to CSV
            command_results[cmd_idx]
                .export_runs_incremental(&time_dir, &mem_dir)
                .with_context(|| {
                    format!("Failed to export run {} for command {}", run_idx, cmd_idx)
                })?;

            Ok(())
        }),
    )
    .map_err(|_| ExitCode::FAILURE)?;

    let _ = measure_memory_for_runs(
        &mem_runs,
        &thread_pool,
        &(&bench_config).into(),
        Some(|run_idx, run_cfg: &RunConfig, run_result: &RunResult| {
            let cmd_idx = run_cfg.cmd_index;

            command_results[cmd_idx].runs.push(run_result.clone());

            // Export immediately to CSV
            command_results[cmd_idx]
                .export_runs_incremental(&time_dir, &mem_dir)
                .with_context(|| {
                    format!(
                        "Failed to export memory run {} for command {}",
                        run_idx, cmd_idx
                    )
                })?;

            Ok(())
        }),
    )
    .map_err(|_| ExitCode::FAILURE)?;

    info!("Benchmarking completed. Results saved incrementally.");

    Ok(())
}

pub(super) fn execute_benchmark(
    bench_config: BenchConfig,
    thread_pool: ThreadPool,
) -> Result<BenchResults, ExitCode> {
    let timing_runs = expand_runs_for_time_measurment(&bench_config);
    let cmd_count = bench_config.commands.len();

    let mut command_results = bench_config
        .commands
        .iter()
        .map(|cmd| CommandResults {
            cmd: cmd.cmd.clone(),
            args: cmd.args.clone(),
            runs: Vec::with_capacity(cmd.iterations.unwrap_or(bench_config.iterations)),
        })
        .collect::<Vec<CommandResults>>();

    let times = measure_execution_time::<fn(usize, &RunConfig, Option<Duration>) -> Result<()>>(
        &timing_runs,
        (&bench_config).into(),
        None,
    )
    .map_err(|_| ExitCode::FAILURE)?;

    for (i, time_result) in times.iter().enumerate() {
        let exec_time_result = ExecutionTimeResult {
            duration: time_result.unwrap_or_default(),
        };
        let command_result = &mut command_results[i % cmd_count];
        command_result.runs.push(RunResult::Time(exec_time_result));
    }

    let mem_runs = expand_runs_for_memory_measurment(&bench_config);
    let memory_stats = measure_memory_for_runs::<fn(usize, &RunConfig, &RunResult) -> Result<()>>(
        &mem_runs,
        &thread_pool,
        &(&bench_config).into(),
        None,
    )
    .map_err(|_| ExitCode::FAILURE)?;
    for (i, mem_stat) in memory_stats.into_iter().enumerate() {
        let cmd_idx = i % cmd_count;
        command_results[cmd_idx].runs.push(mem_stat);
    }

    info!("Benchmarking completed. Preparing results for output.");

    Ok(BenchResults {
        commands: command_results,
    })
}
