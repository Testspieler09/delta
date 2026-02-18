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
#[serde(rename_all = "snake_case")]
pub(super) enum MeasuringMode {
    Timeline,
    Maximum,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
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

    // Warmup Config (can only be set globally)
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::Builder;

    #[test]
    fn test_default_values() {
        assert_eq!(default_iterations(), 200);
        assert_eq!(
            default_threads(),
            std::thread::available_parallelism().unwrap().get()
        );
        assert_eq!(default_max_time(), Duration::from_secs(10));
        assert_eq!(default_max_sampling_interval(), Duration::from_micros(100));
        assert_eq!(default_measure_mem_once(), false);
        assert!(default_memory_measuring_mode() == MeasuringMode::Timeline);
        assert_eq!(default_warmup_count(), 10);
        assert!(default_warmup_mode() == WarmupMode::Global);
    }

    #[test]
    fn test_benchconfig_deserialize_with_defaults() {
        let toml_content = r#"
            command = [{ cmd = "echo", args = ["hello"] }]
        "#;

        let config: BenchConfig = toml::from_str(toml_content).expect("Failed to parse TOML");

        assert_eq!(config.threads, default_threads());
        assert_eq!(config.iterations, default_iterations());
        assert_eq!(config.max_execution_time, default_max_time());
        assert_eq!(
            config.memory_sampling_interval,
            default_max_sampling_interval()
        );
        assert_eq!(config.measure_mem_once, false);
        assert!(config.memory_measuring_mode == MeasuringMode::Timeline);
        assert_eq!(config.warmup_count, default_warmup_count());
        assert!(config.warmup_mode == WarmupMode::Global);

        assert_eq!(config.commands.len(), 1);
        assert_eq!(config.commands[0].cmd, "echo");
        assert_eq!(config.commands[0].args, vec!["hello"]);
    }

    #[test]
    fn test_try_from_pathbuf_valid() {
        let mut file = Builder::new().suffix(".toml").tempfile().unwrap();
        write!(file, "[[command]]\ncmd = \"ls\"\n").unwrap();

        let path = file.path().to_path_buf();
        let config = BenchConfig::try_from(&path).expect("Failed to read config");
        assert_eq!(config.commands[0].cmd, "ls");
    }

    #[test]
    fn test_try_from_pathbuf_invalid_extension() {
        let path = PathBuf::from("config.json");
        let result = BenchConfig::try_from(&path);

        if let Ok(_cfg) = result {
            panic!("Expected an error, got Ok value");
        }

        let err = result.err().unwrap();
        assert!(err.to_string().contains("not a toml file"));
    }

    #[test]
    fn test_into_runconfig_conversion() {
        let mode = MeasuringMode::Maximum;
        let execution_time = Duration::from_secs(5);
        let sampling_interval = Duration::from_micros(50);
        let config = BenchConfig {
            threads: 4,
            iterations: 100,
            max_execution_time: execution_time.clone(),
            memory_sampling_interval: sampling_interval.clone(),
            measure_mem_once: false,
            memory_measuring_mode: mode,
            warmup_count: 3,
            warmup_mode: WarmupMode::Global,
            commands: vec![CommandConfig {
                cmd: "echo".into(),
                args: vec!["hello".into()],
                iterations: None,
                max_execution_time: None,
                memory_sampling_interval: None,
                measure_mem_once: None,
                memory_measuring_mode: None,
            }],
        };

        let run_configs: Vec<RunConfig> = config.into();

        assert_eq!(run_configs.len(), 1);
        let rc = &run_configs[0];
        assert_eq!(rc.cmd, "echo");
        assert_eq!(rc.args, vec!["hello"]);
        assert_eq!(rc.timeout, execution_time);
        assert_eq!(rc.memory_interval, sampling_interval);
        assert!(rc.memory_measuring_mode == mode);
    }
}
