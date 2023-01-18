use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use tracing::debug;

/// The config to be loaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Filename that the database is stored at in the root.
    #[serde(default = "papers_core::db::default_filename")]
    pub db_filename: PathBuf,

    /// Directory of the default repo, if no db found in the parent directories.
    #[serde(default = "default_repo")]
    pub default_repo: PathBuf,
}

fn default_repo() -> PathBuf {
    "~/.local/share/papers".into()
}

impl Config {
    /// Load the config from a file, if it exists.
    /// Returns a default config if the file doesn't exist.
    pub fn load(filename: &Path) -> anyhow::Result<Self> {
        debug!(?filename, "Trying to load config");
        let config = if let Ok(file) = File::open(filename) {
            serde_yaml::from_reader(file)?
        } else {
            serde_yaml::from_str("{}")?
        };
        Ok(config)
    }
}
