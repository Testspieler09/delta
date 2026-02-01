use crate::results::{PeakMemoryResult, TimelineMemoryResult};

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
