use std::collections::BTreeSet;
use std::fs::File;
use std::path::Path;
use std::path::PathBuf;

use directories::ProjectDirs;
use papers_core::label::Label;
use papers_core::tag::Tag;
use serde::Deserialize;
use serde::Serialize;
use tracing::debug;

/// Default values for a paper.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaperDefaults {
    /// Default list of tags.
    #[serde(default)]
    pub tags: BTreeSet<Tag>,
    /// Default list of labels.
    #[serde(default)]
    pub labels: BTreeSet<Label>,
}

/// The config to be loaded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Filename that the database is stored at in the root.
    #[serde(default = "papers_core::db::default_filename")]
    pub db_filename: PathBuf,

    /// Directory of the default repo, if no db found in the parent directories.
    #[serde(default = "default_repo")]
    pub default_repo: PathBuf,

    /// Path to the notes template, either absolute or relative to the `default_repo`.
    pub notes_template: Option<PathBuf>,

    /// Defaults for paper fields on entry
    #[serde(default)]
    pub paper_defaults: PaperDefaults,
}

fn default_repo() -> PathBuf {
    let dirs = ProjectDirs::from("io", "jeffas", "papers").unwrap();
    dirs.data_dir().to_owned()
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
