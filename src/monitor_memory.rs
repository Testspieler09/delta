use crate::{
    bench_config::{MeasuringMode, WarmupMode},
    executor::RunConfig,
    info,
    results::{PeakMemoryResult, RunResult, TimelineMemoryResult},
    warmup::{WarmupConfig, run_warmup},
};

use anyhow::Result;
use nix::sys::{
    resource::{UsageWho, getrusage},
    wait::waitpid,
};
use rayon::{
    ThreadPool,
    iter::{IntoParallelRefIterator, ParallelIterator},
};
use std::{
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

fn monitor_memory(child: &mut Child, sampling_interval: Duration) -> Result<TimelineMemoryResult> {
    let mut s = System::new_all();
    let pid = Pid::from(child.id() as usize);
    let process_refresh_kind = ProcessRefreshKind::nothing().with_memory();

    let start = Instant::now();
    let mut mem_timeline: Vec<(Duration, PeakMemoryResult)> = Vec::with_capacity(2000);

    loop {
        if let Some(_status) = child.try_wait().expect("Check failed") {
            break;
        }

        s.refresh_processes_specifics(ProcessesToUpdate::Some(&[pid]), true, process_refresh_kind);

        if let Some(process) = s.process(pid) {
            let mem_result = PeakMemoryResult {
                physical: process.memory(),
                virtual_: process.virtual_memory(),
            };
            mem_timeline.push((start.elapsed(), mem_result));
        } else {
            break;
        }

        std::thread::sleep(sampling_interval);
    }

    Ok(TimelineMemoryResult {
        timeline: mem_timeline,
    })
}

fn measure_memory_usage_over_time(
    runs: &[RunConfig],
    thread_pool: &ThreadPool,
    warmup_config: &WarmupConfig,
) -> Result<Vec<RunResult>> {
    if warmup_config.iter_count > 0 && warmup_config.mode == WarmupMode::Global {
        run_warmup(runs, &warmup_config);
    }

    let results = thread_pool.install(|| {
        runs.par_iter()
            .map(|run| {
                if warmup_config.mode == WarmupMode::Interval {
                    run_warmup(std::slice::from_ref(run), &warmup_config);
                }

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

fn monitor_virtual_peak_memory(pid_raw: u32, sampling_interval: Duration) -> u64 {
    let mut s = System::new_all();
    let pid = Pid::from(pid_raw as usize);

    let mut max_v_mem = 0;

    loop {
        s.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::everything(),
        );

        if let Some(process) = s.process(pid) {
            let current_v_mem = sysinfo::Process::virtual_memory(process);
            if current_v_mem > max_v_mem {
                max_v_mem = current_v_mem;
            }
        } else {
            break;
        }

        std::thread::sleep(sampling_interval);
    }

    max_v_mem
}

fn run_and_measure_peak_memory(runs: &[RunConfig], warmup_config: &WarmupConfig) -> Vec<RunResult> {
    if warmup_config.iter_count > 0 && warmup_config.mode == WarmupMode::Global {
        run_warmup(runs, warmup_config);
    }

    runs.iter()
        .map(|run| {
            if warmup_config.mode == WarmupMode::Interval {
                run_warmup(std::slice::from_ref(run), warmup_config);
            }

            let before = getrusage(UsageWho::RUSAGE_CHILDREN).unwrap();

            let child = Command::new(&run.cmd)
                .args(&run.args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap();

            let pid = nix::unistd::Pid::from_raw(child.id() as i32);
            let vsz = monitor_virtual_peak_memory(child.id(), run.memory_interval);

            let _ = waitpid(pid, None).unwrap();

            let after = getrusage(UsageWho::RUSAGE_CHILDREN).unwrap();

            let psz = after.max_rss().saturating_sub(before.max_rss()) as u64;

            RunResult::PeakMemory(PeakMemoryResult {
                physical: psz,
                virtual_: vsz,
            })
        })
        .collect()
}

pub(super) fn measure_memory_for_runs(
    runs: &[RunConfig],
    thread_pool: &ThreadPool,
    warmup_config: &WarmupConfig,
) -> Result<Vec<RunResult>> {
    info!("Starting to measure memory usage");
    let (timeline_idxs, max_idxs): (Vec<_>, Vec<_>) = runs
        .iter()
        .enumerate()
        .partition(|(_, run)| run.memory_measuring_mode == MeasuringMode::Timeline);

    let timeline_runs: Vec<RunConfig> = timeline_idxs.into_iter().map(|(_, r)| r.clone()).collect();

    let max_runs: Vec<RunConfig> = max_idxs.into_iter().map(|(_, r)| r.clone()).collect();

    let timeline_results =
        measure_memory_usage_over_time(&timeline_runs, thread_pool, warmup_config);
    let max_results = run_and_measure_peak_memory(&max_runs, warmup_config);

    let mut combined_results = timeline_results?;
    combined_results.extend(max_results);

    Ok(combined_results)
}
