use std::collections::BTreeSet;
use std::fs::File;
use std::io::Read;
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

/// Either a path to a file, or raw content.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PathOrString {
    /// Path to a file.
    File(PathBuf),
    /// Inline content.
    Content(String),
}

impl Default for PathOrString {
    fn default() -> Self {
        Self::Content(String::new())
    }
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
    #[serde(default, with = "serde_yaml::with::singleton_map")]
    pub notes_template: PathOrString,

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
        let file = File::open(filename)?;
        let config = Self::load_reader(file)?;
        Ok(config)
    }

    /// Load the config from a reader.
    pub fn load_reader<R: Read>(r: R) -> anyhow::Result<Self> {
        let config = serde_yaml::from_reader(r)?;
        Ok(config)
    }

    /// Load the config from a string.
    pub fn load_str(s: &str) -> anyhow::Result<Self> {
        let config = Self::load_reader(s.as_bytes())?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};

    use super::*;

    fn check(config: &str, expected: Expect) {
        println!("{}", config);
        let res = Config::load_str(config);
        expected.assert_debug_eq(&res);
    }

    #[test]
    fn test_config_empty() {
        check(
            r#""#,
            expect![[r#"
                Ok(
                    Config {
                        db_filename: "papers.db",
                        default_repo: "/home/andrew/.local/share/papers",
                        notes_template: Content(
                            "",
                        ),
                        paper_defaults: PaperDefaults {
                            tags: {},
                            labels: {},
                        },
                    },
                )
            "#]],
        );
    }

    #[test]
    fn test_config_note_template_path() {
        check(
            r#"notes_template:
  file: some_path.md
"#,
            expect![[r#"
                Ok(
                    Config {
                        db_filename: "papers.db",
                        default_repo: "/home/andrew/.local/share/papers",
                        notes_template: File(
                            "some_path.md",
                        ),
                        paper_defaults: PaperDefaults {
                            tags: {},
                            labels: {},
                        },
                    },
                )
            "#]],
        );
    }

    #[test]
    fn test_config_note_template_inline() {
        check(
            r#"notes_template:
  content: my content
            "#,
            expect![[r#"
                Ok(
                    Config {
                        db_filename: "papers.db",
                        default_repo: "/home/andrew/.local/share/papers",
                        notes_template: Content(
                            "my content",
                        ),
                        paper_defaults: PaperDefaults {
                            tags: {},
                            labels: {},
                        },
                    },
                )
            "#]],
        );
    }

    #[test]
    fn test_config_note_template_inline_multiline() {
        check(
            r#"notes_template:
  content: |
    line 1
    line 2

    a break

     line 3
            "#,
            expect![[r#"
                Ok(
                    Config {
                        db_filename: "papers.db",
                        default_repo: "/home/andrew/.local/share/papers",
                        notes_template: Content(
                            "line 1\nline 2\n\na break\n\n line 3\n        ",
                        ),
                        paper_defaults: PaperDefaults {
                            tags: {},
                            labels: {},
                        },
                    },
                )
            "#]],
        );
    }
}
