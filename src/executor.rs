use crate::{
    bench_config::BenchConfig,
    info,
    monitor_memory::monitor_memory,
    parser::MeasuringMode,
    results::{
        BenchResults, CommandResults, ExecutionTimeResult, PeakMemoryResult, RunResult,
        TimelineMemoryResult,
    },
};

use anyhow::Result;
use nix::{
    sys::{
        resource::{UsageWho, getrusage},
        wait::waitpid,
    },
    unistd::Pid,
};
use rayon::{ThreadPool, prelude::*};
use std::{
    process::{Command, ExitCode, Stdio},
    time::{Duration, Instant},
};
use wait_timeout::ChildExt;

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

pub(super) fn measure_execution_time(
    runs: &[RunConfig],
    thread_pool: &ThreadPool,
) -> Result<Vec<Option<Duration>>> {
    info!("Starting to measure execution times");

    let results = thread_pool.install(|| {
        runs.par_iter()
            .map(|run| {
                let mut child = Command::new(&run.cmd)
                    .args(&run.args)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .ok()?;

                let start = Instant::now();

                match child.wait_timeout(run.timeout).ok()? {
                    Some(_) => Some(start.elapsed()),
                    None => {
                        let _ = child.kill();
                        let _ = child.wait();
                        None
                    }
                }
            })
            .collect()
    });

    info!("Finished measuring execution times");
    Ok(results)
}

pub(super) fn measure_memory_usage_over_time(
    runs: &[RunConfig],
    thread_pool: &ThreadPool,
) -> Result<Vec<RunResult>> {
    let results = thread_pool.install(|| {
        runs.par_iter()
            .map(|run| {
                let mut child = Command::new(&run.cmd)
                    .args(&run.args)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .expect("spawn failed");

                let mem_result = match monitor_memory(&mut child, run.memory_interval) {
                    Ok(result) => Some(RunResult::TimelineMemory(result)),
                    Err(_) => None,
                };

                let _ = child.wait();
                mem_result
            })
            .filter_map(|r| r)
            .collect()
    });

    Ok(results)
}

pub fn run_and_measure_peak_memory(runs: &[RunConfig]) -> Vec<RunResult> {
    runs.iter()
        .map(|run| {
            let before = getrusage(UsageWho::RUSAGE_CHILDREN).unwrap();

            let child = Command::new(&run.cmd)
                .args(&run.args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();

            let pid = Pid::from_raw(child.id() as i32);
            let _ = waitpid(pid, None).unwrap();

            let after = getrusage(UsageWho::RUSAGE_CHILDREN).unwrap();

            let psz = after.max_rss().saturating_sub(before.max_rss()) as u64;

            RunResult::PeakMemory(PeakMemoryResult {
                physical: psz,
                virtual_: 0, // TODO: try via polling inside a different fn?
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
