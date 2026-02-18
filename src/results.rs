use anyhow::Result;
use csv::Writer;
use std::{fs, path::PathBuf, time::Duration};

/// Peak memory per run
#[derive(Clone, Copy)]
pub struct PeakMemoryResult {
    pub physical: u64,
    pub virtual_: u64,
}

/// Timeline memory per run
#[derive(Clone)]
pub struct TimelineMemoryResult {
    pub timeline: Vec<(Duration, PeakMemoryResult)>,
}

/// Execution time per run
#[derive(Clone, Copy)]
pub struct ExecutionTimeResult {
    pub duration: Duration,
}

/// A single run result, which can be different types depending on user choice
#[derive(Clone)]
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
        let time_dir = output_folder.join("time");
        let mem_dir = output_folder.join("memory");

        fs::create_dir_all(&time_dir)?;
        fs::create_dir_all(&mem_dir)?;

        for (cmd_idx, cmd_res) in self.commands.iter().enumerate() {
            let safe_base_name = format!(
                "{}_{}_{}",
                cmd_idx,
                cmd_res.cmd.replace(|c: char| !c.is_alphanumeric(), "_"),
                cmd_res
                    .args
                    .join("_")
                    .replace(|c: char| !c.is_alphanumeric(), "_"),
            );

            let mut time_wtr = None;
            let mut peak_wtr = None;
            let mut tl_wtr = None;

            for (run_idx, run) in cmd_res.runs.iter().enumerate() {
                match run {
                    RunResult::Time(t) => {
                        let wtr = if let Some(ref mut w) = time_wtr {
                            w
                        } else {
                            let path = time_dir.join(format!("{}_time.csv", safe_base_name));
                            let mut w = Writer::from_path(path)?;
                            w.write_record(&["run_index", "duration_ns"])?;
                            time_wtr = Some(w);
                            time_wtr.as_mut().unwrap()
                        };
                        wtr.write_record(&[
                            run_idx.to_string(),
                            t.duration.as_nanos().to_string(),
                        ])?;
                    }
                    RunResult::PeakMemory(m) => {
                        let wtr = if let Some(ref mut w) = peak_wtr {
                            w
                        } else {
                            let path = mem_dir.join(format!("{}_peak.csv", safe_base_name));
                            let mut w = Writer::from_path(path)?;
                            w.write_record(&["run_index", "physical_bytes", "virtual_bytes"])?;
                            peak_wtr = Some(w);
                            peak_wtr.as_mut().unwrap()
                        };
                        wtr.write_record(&[
                            run_idx.to_string(),
                            m.physical.to_string(),
                            m.virtual_.to_string(),
                        ])?;
                    }
                    RunResult::TimelineMemory(tm) => {
                        let wtr = if let Some(ref mut w) = tl_wtr {
                            w
                        } else {
                            let path = mem_dir.join(format!("{}_timeline.csv", safe_base_name));
                            let mut w = Writer::from_path(path)?;
                            w.write_record(&[
                                "run_index",
                                "timestamp_ms",
                                "physical_bytes",
                                "virtual_bytes",
                            ])?;
                            tl_wtr = Some(w);
                            tl_wtr.as_mut().unwrap()
                        };
                        for (timestamp, stats) in &tm.timeline {
                            wtr.write_record(&[
                                run_idx.to_string(),
                                timestamp.as_millis().to_string(),
                                stats.physical.to_string(),
                                stats.virtual_.to_string(),
                            ])?;
                        }
                    }
                }
            }

            if let Some(mut w) = time_wtr {
                w.flush()?;
            }
            if let Some(mut w) = peak_wtr {
                w.flush()?;
            }
            if let Some(mut w) = tl_wtr {
                w.flush()?;
            }
        }

        Ok(())
    }
}
