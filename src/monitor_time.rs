use crate::{executor::RunConfig, info};

use anyhow::Result;
use rayon::{
    ThreadPool,
    iter::{IntoParallelRefIterator, ParallelIterator},
};
use std::{
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use wait_timeout::ChildExt;

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
