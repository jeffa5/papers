use std::fs::File;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {}

impl Config {
    pub fn load(filename: &Path) -> anyhow::Result<Self> {
        let config = if let Ok(file) = File::open(filename) {
            serde_yaml::from_reader(file)?
        } else {
            serde_yaml::from_str("{}")?
        };
        Ok(config)
    }
}
