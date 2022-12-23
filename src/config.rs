use std::fs::File;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {}

impl Config {
    pub fn load(filename: &Path) -> Self {
        if let Ok(file) = File::open(filename) {
            serde_yaml::from_reader(file).unwrap()
        } else {
            serde_yaml::from_str("{}").unwrap()
        }
    }
}
