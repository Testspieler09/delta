use crate::{
    bench_config::WarmupMode,
    executor::RunConfig,
    info,
    warmup::{WarmupConfig, run_warmup},
};

use anyhow::{Result, anyhow};
use std::{
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use wait_timeout::ChildExt;

pub(super) fn measure_execution_time<F>(
    runs: &[RunConfig],
    warmup_config: WarmupConfig,
    mut per_run_callback: Option<F>,
) -> Result<Vec<Option<Duration>>>
where
    F: FnMut(usize, &RunConfig, Option<Duration>) -> Result<()>,
{
    if warmup_config.iter_count > 0 && warmup_config.mode == WarmupMode::Global {
        run_warmup(runs, &warmup_config);
    }

    info!("Starting to measure execution times");

    let mut results = Vec::with_capacity(runs.len());

    for (i, run) in runs.iter().enumerate() {
        info!("Running time measurement {} {:?}", run.cmd, run.args);

        if warmup_config.mode == WarmupMode::Interval {
            run_warmup(std::slice::from_ref(run), &warmup_config);
        }

        let mut child = match Command::new(&run.cmd)
            .args(&run.args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => {
                results.push(None);
                if let Some(cb) = per_run_callback.as_mut() {
                    cb(i, run, None)?;
                }
                continue;
            }
        };

        let start = Instant::now();
        let elapsed = match child
            .wait_timeout(run.timeout)
            .map_err(|e| anyhow!("Failed to wait on child: {:?}", e))?
        {
            Some(_) => Some(start.elapsed()),
            None => {
                let _ = child.kill();
                let _ = child.wait();
                info!("Killed process as it exceeded timeout");
                None
            }
        };

        results.push(elapsed);

        if let Some(cb) = per_run_callback.as_mut() {
            cb(i, run, elapsed)?;
        }
    }

    Ok(results)
}
