use crate::{
    bench_config::{BenchConfig, MeasuringMode, WarmupMode},
    info,
    monitor_memory::measure_memory_for_runs,
    monitor_time::measure_execution_time,
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
    pub memory_measuring_mode: MeasuringMode,
    pub warmup_count: usize,
    pub warmup_mode: WarmupMode,
}

fn expand_runs_for_time_measurment(bench_config: &BenchConfig) -> Vec<RunConfig> {
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
                memory_measuring_mode: cmd
                    .memory_measuring_mode
                    .unwrap_or(bench_config.memory_measuring_mode),
                warmup_count: cmd.warmup_count.unwrap_or(bench_config.warmup_count),
                warmup_mode: cmd.warmup_mode.unwrap_or(bench_config.warmup_mode),
            })
        })
        .collect()
}

fn expand_runs_for_memory_measurment(bench_config: &BenchConfig) -> Vec<RunConfig> {
    bench_config
        .commands
        .iter()
        .flat_map(|cmd| {
            let iters = if cmd
                .measure_mem_once
                .unwrap_or(bench_config.measure_mem_once)
            {
                1
            } else {
                cmd.iterations.unwrap_or(bench_config.iterations)
            };

            (0..iters).map(move |_| RunConfig {
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
                warmup_count: cmd.warmup_count.unwrap_or(bench_config.warmup_count),
                warmup_mode: cmd.warmup_mode.unwrap_or(bench_config.warmup_mode),
            })
        })
        .collect()
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
            runs: Vec::with_capacity(2),
        })
        .collect::<Vec<CommandResults>>();

    let times = measure_execution_time(&timing_runs).map_err(|_| ExitCode::FAILURE)?;

    for (i, time_result) in times.iter().enumerate() {
        let exec_time_result = ExecutionTimeResult {
            duration: time_result.unwrap_or_default(),
        };
        let command_result = &mut command_results[i % cmd_count];
        command_result.runs.push(RunResult::Time(exec_time_result));
    }

    let mem_runs = expand_runs_for_memory_measurment(&bench_config);
    let memory_stats =
        measure_memory_for_runs(&mem_runs, &thread_pool).map_err(|_| ExitCode::FAILURE)?;
    for (i, mem_stat) in memory_stats.into_iter().enumerate() {
        let cmd_idx = i % cmd_count;
        command_results[cmd_idx].runs.push(mem_stat);
    }

    info!("Benchmarking completed. Preparing results for output.");

    Ok(BenchResults {
        commands: command_results,
    })
}
