use anyhow::Result;
use std::{process::Child, time::Duration};
use sysinfo::{Pid, System};

pub(super) fn monitor_memory(child: &mut Child, sampling_interval: Duration) -> Result<u64> {
    let s = System::new_all();
    let process = s
        .process(Pid::from(child.id() as usize))
        .expect("Process not running nomore");

    let mut max_mem = 0;

    loop {
        if let Some(_status) = child.try_wait().expect("Check failed") {
            break;
        }

        let current_mem = sysinfo::Process::memory(process);
        if current_mem > max_mem {
            max_mem = current_mem;
        }

        std::thread::sleep(sampling_interval);
    }

    Ok(max_mem)
}
