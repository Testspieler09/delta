use crate::{
    bench_config::{BenchConfig, WarmupMode},
    executor::RunConfig,
};
use std::process::{Command, Stdio};
use wait_timeout::ChildExt;

pub struct WarmupConfig {
    pub mode: WarmupMode,
    pub iter_count: usize,
}

impl From<&BenchConfig> for WarmupConfig {
    fn from(config: &BenchConfig) -> Self {
        Self {
            mode: config.warmup_mode,
            iter_count: config.warmup_count,
        }
    }
}

pub(super) fn run_warmup(runs: &[RunConfig], config: &WarmupConfig) {
    if config.iter_count == 0 {
        return;
    }

    for _ in 0..config.iter_count {
        for run in runs {
            let mut child = match Command::new(&run.cmd)
                .args(&run.args)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(child) => child,
                Err(_) => continue,
            };

            match child.wait_timeout(run.timeout) {
                Ok(Some(_)) => {}
                Ok(None) => {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                Err(_) => {}
            }
        }
    }
}
