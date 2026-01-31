use anyhow::{Result, anyhow};
use serde::Deserialize;
use std::{fs, path::PathBuf, time::Duration};

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

#[derive(Deserialize, Debug)]
pub struct BenchConfig {
    #[serde(default = "default_threads")]
    pub threads: usize,

    #[serde(default = "default_iterations")]
    pub iterations: usize,

    #[serde(default = "default_max_time")]
    #[serde(with = "humantime_serde")]
    pub max_execution_time: Duration,

    #[serde(default = "default_max_sampling_interval")]
    #[serde(with = "humantime_serde")]
    pub memory_sampling_interval: Duration,

    #[serde(rename = "command")]
    pub commands: Vec<CommandConfig>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct CommandConfig {
    pub cmd: String,

    #[serde(default)]
    pub args: Vec<String>,

    #[serde(default)]
    pub iterations: Option<usize>,

    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub max_execution_time: Option<Duration>,

    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub memory_sampling_interval: Option<Duration>,
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
