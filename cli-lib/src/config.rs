use std::fs::File;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

/// The config to be loaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {}

impl Config {
    /// Load the config from a file, if it exists.
    /// Returns a default config if the file doesn't exist.
    pub fn load(filename: &Path) -> anyhow::Result<Self> {
        let config = if let Ok(file) = File::open(filename) {
            serde_yaml::from_reader(file)?
        } else {
            serde_yaml::from_str("{}")?
        };
        Ok(config)
    }
}