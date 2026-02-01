use anyhow::{Result, anyhow};
use std::{path::PathBuf, time::Duration};

/// Peak memory per run
pub struct PeakMemoryResult {
    pub physical: u64,
    pub virtual_: u64,
}

/// Timeline memory per run
pub struct TimelineMemoryResult {
    pub timeline: Vec<(Duration, PeakMemoryResult)>,
}

/// Execution time per run
pub struct ExecutionTimeResult {
    pub duration: Duration,
}

/// A single run result, which can be different types depending on user choice
pub enum RunResult {
    Time(ExecutionTimeResult),
    PeakMemory(PeakMemoryResult),
    TimelineMemory(TimelineMemoryResult),
}

pub struct CommandResults {
    pub cmd: String,
    pub args: Vec<String>,
    pub runs: Vec<RunResult>,
}

pub struct BenchResults {
    pub commands: Vec<CommandResults>,
}

impl BenchResults {
    pub fn export_to_csv_files(&self, output_folder: PathBuf) -> Result<()> {
        if !output_folder.is_dir() {
            return Err(anyhow!(
                "{:?} does not exists or is not a directory.",
                output_folder
            ));
        }

        Ok(())
    }
}
