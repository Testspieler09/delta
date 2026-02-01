use crate::{
    executor::RunConfig,
    results::{PeakMemoryResult, RunResult, TimelineMemoryResult},
};
use nix::sys::{
    resource::{UsageWho, getrusage},
    wait::waitpid,
};
use rayon::{
    ThreadPool,
    iter::{IntoParallelRefIterator, ParallelIterator},
};
use std::process::{Command, Stdio};

use anyhow::Result;
use std::{
    process::Child,
    time::{Duration, Instant},
};
use sysinfo::{Pid, System};

pub(super) fn monitor_memory(
    child: &mut Child,
    sampling_interval: Duration,
) -> Result<TimelineMemoryResult> {
    let s = System::new_all();
    let process = s
        .process(Pid::from(child.id() as usize))
        .expect("Process not running nomore");

    let start = Instant::now();
    let mut mem_timeline: Vec<(Duration, PeakMemoryResult)> = Vec::with_capacity(2000);

    loop {
        if let Some(_status) = child.try_wait().expect("Check failed") {
            break;
        }

        let mem_result = PeakMemoryResult {
            physical: sysinfo::Process::memory(process),
            virtual_: sysinfo::Process::virtual_memory(process),
        };

        mem_timeline.push((start.elapsed(), mem_result));

        std::thread::sleep(sampling_interval);
    }

    Ok(TimelineMemoryResult {
        timeline: mem_timeline,
    })
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

            let pid = nix::unistd::Pid::from_raw(child.id() as i32);
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
