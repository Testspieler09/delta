use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::{fs, path::PathBuf, time::Duration};

use crate::executor::RunConfig;

fn default_threads() -> usize {
    std::thread::available_parallelism()
        .expect("Could not get number of availabe cores")
        .get()
}
fn default_iterations() -> usize {
    200
}
fn default_max_time() -> Duration {
    Duration::from_secs(10)
}
fn default_max_sampling_interval() -> Duration {
    Duration::from_micros(100)
}
fn default_measure_mem_once() -> bool {
    false
}
fn default_memory_measuring_mode() -> MeasuringMode {
    MeasuringMode::Timeline
}
fn default_warmup_count() -> usize {
    10
}
fn default_warmup_mode() -> WarmupMode {
    WarmupMode::Global
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
pub(super) enum MeasuringMode {
    Timeline,
    Maximum,
}

#[derive(Deserialize, Clone, Copy)]
pub enum WarmupMode {
    /// Run warmup with all cmds once before the cmds
    Global,

    /// Run warmup for each cmd right before it (best case measurments)
    Interval,
}

#[derive(Deserialize)]
pub struct BenchConfig {
    // General Config
    #[serde(default = "default_threads")]
    pub threads: usize,

    #[serde(default = "default_iterations")]
    pub iterations: usize,

    // Time measurment Config
    #[serde(default = "default_max_time")]
    #[serde(with = "humantime_serde")]
    pub max_execution_time: Duration,

    // Memory measurment Config
    #[serde(default = "default_max_sampling_interval")]
    #[serde(with = "humantime_serde")]
    pub memory_sampling_interval: Duration,

    #[serde(default = "default_measure_mem_once")]
    pub measure_mem_once: bool,

    #[serde(default = "default_memory_measuring_mode")]
    pub memory_measuring_mode: MeasuringMode,

    // Warmup Config
    #[serde(default = "default_warmup_count")]
    pub warmup_count: usize,

    #[serde(default = "default_warmup_mode")]
    pub warmup_mode: WarmupMode,

    #[serde(rename = "command")]
    pub commands: Vec<CommandConfig>,
}

impl Into<Vec<RunConfig>> for BenchConfig {
    fn into(self) -> Vec<RunConfig> {
        self.commands
            .iter()
            .map(|c| RunConfig {
                cmd: c.cmd.clone(),
                args: c.args.clone(),
                timeout: c.max_execution_time.unwrap_or(self.max_execution_time),
                memory_interval: c
                    .memory_sampling_interval
                    .unwrap_or(self.memory_sampling_interval),
                memory_measuring_mode: c
                    .memory_measuring_mode
                    .unwrap_or(self.memory_measuring_mode),
                warmup_count: c.warmup_count.unwrap_or(self.warmup_count),
                warmup_mode: c.warmup_mode.unwrap_or(self.warmup_mode),
            })
            .collect()
    }
}

#[derive(Deserialize, Clone)]
pub struct CommandConfig {
    // General Config
    pub cmd: String,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub iterations: Option<usize>,

    // Time measurment Config
    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub max_execution_time: Option<Duration>,

    // Memory measurment Config
    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub memory_sampling_interval: Option<Duration>,

    #[serde(default)]
    pub measure_mem_once: Option<bool>,

    #[serde(default)]
    pub memory_measuring_mode: Option<MeasuringMode>,

    // Warmup Config
    #[serde(default)]
    pub warmup_count: Option<usize>,

    #[serde(default)]
    pub warmup_mode: Option<WarmupMode>,
}

impl TryFrom<&PathBuf> for BenchConfig {
    type Error = anyhow::Error;

    fn try_from(path: &PathBuf) -> Result<Self> {
        if path.extension().and_then(|s| s.to_str()) != Some("toml") {
            return Err(anyhow!("The config file is not a toml file."));
        }

        let content = fs::read_to_string(&path)?;

        let config: Self =
            toml::from_str(&content).map_err(|e| anyhow!("Failed to parse TOML: {}", e))?;

        Ok(config)
    }
}
