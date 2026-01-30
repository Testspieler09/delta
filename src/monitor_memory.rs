use anyhow::Result;
use std::{
    process::Child,
    time::{Duration, Instant},
};
use sysinfo::{Pid, System};

pub(super) fn monitor_memory(child: &mut Child) -> Result<(Duration, u64)> {
    let s = System::new_all();
    let process = s
        .process(Pid::from(child.id() as usize))
        .expect("Process not running nomore");

    let start = Instant::now();
    let mut max_mem = 0;

    loop {
        if let Some(_status) = child.try_wait().expect("Check failed") {
            break;
        }

        let current_mem = sysinfo::Process::memory(process);
        if current_mem > max_mem {
            max_mem = current_mem;
        }

        // TODO: add this as param to the config
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let duration = start.elapsed();

    Ok((duration, max_mem))
}
