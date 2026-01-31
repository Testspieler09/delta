use crate::{bench_config::BenchConfig, monitor_memory::monitor_memory};
use std::{
    process::{Command, ExitCode, Stdio},
    sync::{Arc, Mutex},
    thread,
    time::Instant,
};

pub(super) fn execute_benchmark(bench_config: BenchConfig) -> Result<(), ExitCode> {
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::with_capacity(bench_config.threads);

    for cmd_config in bench_config.commands {
        for _ in 0..bench_config.threads {
            let cmd_config = cmd_config.clone();
            let results = Arc::clone(&results);

            let handle = thread::spawn(move || {
                let mut child = match Command::new(&cmd_config.cmd)
                    .args(&cmd_config.args)
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                {
                    Ok(child) => child,
                    Err(e) => {
                        eprintln!(
                            "Could not spawn process with cmd: {:?} {:?}\nError: {}",
                            &cmd_config.cmd, &cmd_config.args, e
                        );
                        return Err(ExitCode::FAILURE);
                    }
                };

                let start = Instant::now();

                let sampling_interval = cmd_config
                    .memory_sampling_interval
                    .unwrap_or(bench_config.memory_sampling_interval);

                match monitor_memory(&mut child, sampling_interval) {
                    Ok(max_mem) => {
                        let _status = child
                            .wait()
                            .expect(&format!("Command {:?} was not running", &cmd_config.cmd));
                        let duration = start.elapsed();

                        let mut results_lock = results.lock().unwrap();
                        results_lock.push((
                            cmd_config.cmd.clone(),
                            cmd_config.args,
                            duration,
                            max_mem,
                        ));
                    }
                    Err(_) => {
                        eprintln!("Failed to read process memory.");
                        return Err(ExitCode::FAILURE);
                    }
                }

                Ok(())
            });
            handles.push(handle);
        }
    }

    for handle in handles {
        let _ = handle.join().expect("Thread crashed.");
    }

    let results_lock = results.lock().unwrap();
    for (cmd, args, duration, max_mem) in results_lock.iter() {
        println!(
            "{:?} {:?} ran in {:?} and used {} bytes of memory",
            cmd, args, duration, max_mem
        );
    }

    Ok(())
}
