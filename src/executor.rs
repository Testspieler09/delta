use crate::{bench_config::BenchConfig, monitor_memory::monitor_memory};

use rayon::ThreadPool;
use std::{
    process::{Command, ExitCode, Stdio},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};
use wait_timeout::ChildExt;

pub(super) fn execute_benchmark(
    bench_config: BenchConfig,
    thread_pool: ThreadPool,
) -> Result<(), ExitCode> {
    thread_pool.install(|| {
        use rayon::prelude::*;

        let results: Vec<_> = bench_config
            .commands
            .par_iter()
            .flat_map(|cmd_config| {
                let iterations = cmd_config.iterations.unwrap_or(bench_config.iterations);
                (0..iterations).into_par_iter().map(move |_| cmd_config)
            })
            .map(|cmd_config| {
                let mut child = Command::new(&cmd_config.cmd)
                    .args(&cmd_config.args)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .map_err(|e| {
                        eprintln!(
                            "Could not spawn process with cmd: {:?} {:?}\nError: {}",
                            &cmd_config.cmd, &cmd_config.args, e
                        );
                        ExitCode::FAILURE
                    })?;

                let start = Instant::now();
                let timeout = cmd_config
                    .max_execution_time
                    .unwrap_or(bench_config.max_execution_time);
                let interval = cmd_config
                    .memory_sampling_interval
                    .unwrap_or(bench_config.memory_sampling_interval);

                let result = std::thread::scope(|s| {
                    let max_mem = Arc::new(AtomicU64::new(0));
                    let max_mem_clone = max_mem.clone();

                    let pid = child.id();
                    s.spawn(move || {
                        if let Ok(mem) = monitor_memory(pid, interval) {
                            max_mem_clone.store(mem as u64, Ordering::Relaxed);
                        }
                    });

                    match child.wait_timeout(timeout).unwrap() {
                        Some(_status) => (Some(start.elapsed()), max_mem.load(Ordering::Relaxed)),
                        None => {
                            let _ = child.kill();
                            let _ = child.wait();
                            eprintln!("Killed process {:?} due to timeout", cmd_config.cmd);
                            (None, max_mem.load(Ordering::Relaxed))
                        }
                    }
                });

                let (duration, max_mem) = result;
                Ok((
                    cmd_config.cmd.clone(),
                    cmd_config.args.clone(),
                    duration,
                    max_mem,
                ))
            })
            .collect::<Result<Vec<_>, ExitCode>>()?;

        for (cmd, args, duration, max_mem) in results {
            println!(
                "{:?} {:?} ran in {:?} and used {} bytes of memory",
                cmd, args, duration, max_mem
            );
        }

        Ok(())
    })
}
