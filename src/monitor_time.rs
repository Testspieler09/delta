use crate::{
    bench_config::WarmupMode,
    executor::RunConfig,
    info,
    warmup::{WarmupConfig, run_warmup},
};

use anyhow::Result;
use std::{
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use wait_timeout::ChildExt;

pub(super) fn measure_execution_time(
    runs: &[RunConfig],
    warmup_config: WarmupConfig,
) -> Result<Vec<Option<Duration>>> {
    if warmup_config.iter_count > 0 && warmup_config.mode == WarmupMode::Global {
        run_warmup(runs, &warmup_config);
    }

    info!("Starting to measure execution times");
    let results = runs
        .iter()
        .map(|run| {
            info!("Running time measurement {} {:?}", run.cmd, run.args);
            if warmup_config.mode == WarmupMode::Interval {
                run_warmup(std::slice::from_ref(run), &warmup_config);
            }

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
                    info!("Killed process as it exceeded max_execution_time");
                    let _ = child.kill();
                    let _ = child.wait();
                    None
                }
            }
        })
        .collect();

    Ok(results)
}
