use anyhow::Result;
use std::time::Duration;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};

pub(super) fn monitor_memory(pid_raw: u32, sampling_interval: Duration) -> Result<u64> {
    let mut s = System::new_all();
    let pid = Pid::from(pid_raw as usize);

    let mut max_mem = 0;

    loop {
        s.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::everything(),
        );

        if let Some(process) = s.process(pid) {
            let current_mem = sysinfo::Process::memory(process);
            let _current_v_mem = sysinfo::Process::virtual_memory(process);
            if current_mem > max_mem {
                max_mem = current_mem;
            }
        } else {
            break;
        }

        std::thread::sleep(sampling_interval);
    }

    Ok(max_mem)
}
