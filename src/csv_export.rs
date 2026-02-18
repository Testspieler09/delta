use crate::results::RunResult;

use anyhow::Result;
use csv::Writer;
use std::path::PathBuf;

pub struct CommandCsvWriters {
    pub time_wtr: Writer<std::fs::File>,
    pub peak_wtr: Writer<std::fs::File>,
    pub timeline_wtr: Writer<std::fs::File>,
}

impl CommandCsvWriters {
    pub fn new(
        cmd_idx: usize,
        cmd: &str,
        args: &[String],
        time_dir: &PathBuf,
        mem_dir: &PathBuf,
    ) -> Result<Self> {
        let safe_base_name = format!(
            "{}_{}_{}",
            cmd_idx,
            cmd.replace(|c: char| !c.is_alphanumeric(), "_"),
            args.join("_").replace(|c: char| !c.is_alphanumeric(), "_"),
        );

        let mut time_wtr =
            Writer::from_path(time_dir.join(format!("{}_time.csv", safe_base_name)))?;
        time_wtr.write_record(&["run_index", "duration_ns"])?;

        let mut peak_wtr = Writer::from_path(mem_dir.join(format!("{}_peak.csv", safe_base_name)))?;
        peak_wtr.write_record(&["run_index", "physical_bytes", "virtual_bytes"])?;

        let mut timeline_wtr =
            Writer::from_path(mem_dir.join(format!("{}_timeline.csv", safe_base_name)))?;
        timeline_wtr.write_record(&[
            "run_index",
            "timestamp_ms",
            "physical_bytes",
            "virtual_bytes",
        ])?;

        Ok(Self {
            time_wtr,
            peak_wtr,
            timeline_wtr,
        })
    }

    pub fn write_run(&mut self, run_idx: usize, run: &RunResult) -> Result<()> {
        match run {
            RunResult::Time(t) => {
                self.time_wtr
                    .write_record(&[run_idx.to_string(), t.duration.as_nanos().to_string()])?;
            }
            RunResult::PeakMemory(m) => {
                self.peak_wtr.write_record(&[
                    run_idx.to_string(),
                    m.physical.to_string(),
                    m.virtual_.to_string(),
                ])?;
            }
            RunResult::TimelineMemory(tm) => {
                for (timestamp, stats) in &tm.timeline {
                    self.timeline_wtr.write_record(&[
                        run_idx.to_string(),
                        timestamp.as_millis().to_string(),
                        stats.physical.to_string(),
                        stats.virtual_.to_string(),
                    ])?;
                }
            }
        }
        Ok(())
    }

    pub fn flush_all(&mut self) -> Result<()> {
        self.time_wtr.flush()?;
        self.peak_wtr.flush()?;
        self.timeline_wtr.flush()?;
        Ok(())
    }
}
