use anyhow::Result;
use csv::Writer;
use std::{
    fs::{self, File},
    path::PathBuf,
    time::Duration,
};

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

impl RunResult {
    pub fn export_to_csv_file(&self, writer: &mut Writer<File>, run_idx: usize) -> Result<()> {
        match self {
            RunResult::Time(t) => {
                writer.write_record(&[run_idx.to_string(), t.duration.as_nanos().to_string()])?;
            }
            RunResult::PeakMemory(m) => {
                writer.write_record(&[
                    run_idx.to_string(),
                    m.physical.to_string(),
                    m.virtual_.to_string(),
                ])?;
            }
            RunResult::TimelineMemory(tm) => {
                for (timestamp, stats) in &tm.timeline {
                    writer.write_record(&[
                        run_idx.to_string(),
                        timestamp.as_millis().to_string(),
                        stats.physical.to_string(),
                        stats.virtual_.to_string(),
                    ])?;
                }
            }
        }
        writer.flush()?;
        Ok(())
    }
}

pub struct CommandResults {
    pub cmd: String,
    pub args: Vec<String>,
    pub runs: Vec<RunResult>,
}

impl CommandResults {
    /// Generate a sanitized base filename for CSVs
    fn safe_base_name(&self) -> String {
        format!(
            "{}_{}",
            self.cmd.replace(|c: char| !c.is_alphanumeric(), "_"),
            self.args
                .join("_")
                .replace(|c: char| !c.is_alphanumeric(), "_")
        )
    }

    /// Helper to get or create a CSV writer
    fn get_writer(path: &PathBuf, headers: &[&str]) -> Result<Writer<File>> {
        let mut wtr = Writer::from_path(path)?;
        wtr.write_record(headers)?;
        Ok(wtr)
    }

    /// Export runs incrementally (flushes after each run)
    pub fn export_runs_incremental(&self, time_dir: &PathBuf, mem_dir: &PathBuf) -> Result<()> {
        let safe_base = self.safe_base_name();

        let mut time_writer: Option<Writer<File>> = None;
        let mut peak_writer: Option<Writer<File>> = None;
        let mut timeline_writer: Option<Writer<File>> = None;

        for (i, run) in self.runs.iter().enumerate() {
            match run {
                RunResult::Time(_) => {
                    let wtr = time_writer.get_or_insert_with(|| {
                        Self::get_writer(
                            &time_dir.join(format!("{}_time.csv", safe_base)),
                            &["run_index", "duration_ns"],
                        )
                        .unwrap()
                    });
                    run.export_to_csv_file(wtr, i)?;
                }
                RunResult::PeakMemory(_) => {
                    let wtr = peak_writer.get_or_insert_with(|| {
                        Self::get_writer(
                            &mem_dir.join(format!("{}_peak.csv", safe_base)),
                            &["run_index", "physical_bytes", "virtual_bytes"],
                        )
                        .unwrap()
                    });
                    run.export_to_csv_file(wtr, i)?;
                }
                RunResult::TimelineMemory(_) => {
                    let wtr = timeline_writer.get_or_insert_with(|| {
                        Self::get_writer(
                            &mem_dir.join(format!("{}_timeline.csv", safe_base)),
                            &[
                                "run_index",
                                "timestamp_ms",
                                "physical_bytes",
                                "virtual_bytes",
                            ],
                        )
                        .unwrap()
                    });
                    run.export_to_csv_file(wtr, i)?;
                }
            }
        }

        Ok(())
    }
}

pub struct BenchResults {
    pub commands: Vec<CommandResults>,
}

impl BenchResults {
    /// Export all commands, DRYed using helpers
    pub fn export_to_csv_files(&self, output_folder: PathBuf) -> Result<()> {
        let time_dir = output_folder.join("time");
        let mem_dir = output_folder.join("memory");

        fs::create_dir_all(&time_dir)?;
        fs::create_dir_all(&mem_dir)?;

        for cmd_res in &self.commands {
            cmd_res.export_runs_incremental(&time_dir, &mem_dir)?;
        }

        Ok(())
    }
}
