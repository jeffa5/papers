use std::time::Duration;

use cli_table::Table;
use papers_core::{author::Author, label::Label, paper::Paper, tag::Tag};
use serde::Serialize;

/// Paper format for display in a table.
#[derive(Debug, Table, Serialize)]
pub struct TablePaper {
    /// Id of the paper.
    pub id: i32,
    /// Url the paper was fetched from.
    #[table(display_fn = "display_optional_string")]
    pub url: Option<String>,
    /// Local filename of the document.
    pub filename: String,
    /// Title of the document.
    #[table(display_fn = "display_optional_string")]
    pub title: Option<String>,
    /// Tags for this document.
    #[table(display_fn = "display_tag_vector")]
    pub tags: Vec<Tag>,
    /// Labels for this document.
    #[table(display_fn = "display_label_vector")]
    pub labels: Vec<Label>,
    /// Authors for this document.
    #[table(display_fn = "display_author_vector")]
    pub authors: Vec<Author>,
    /// Age since creation.
    #[table(display_fn = "display_duration")]
    pub age: Duration,
}

fn display_duration(dur: &Duration) -> String {
    let mut secs = dur.as_secs();
    if secs < 60 {
        format!("{secs}s")
    } else if secs < 60 * 60 {
        secs /= 60;
        format!("{secs}m")
    } else if secs < 60 * 60 * 24 {
        secs /= 60 * 60;
        format!("{secs}h")
    } else {
        secs /= 60 * 60 * 24;
        format!("{secs}d")
    }
}

fn display_author_vector(v: &[Author]) -> String {
    v.iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(", ")
}

fn display_optional_string(s: &Option<String>) -> String {
    if let Some(s) = s {
        s.clone()
    } else {
        String::new()
    }
}

fn display_tag_vector(v: &[Tag]) -> String {
    v.iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

fn display_label_vector(v: &[Label]) -> String {
    v.iter()
        .map(std::string::ToString::to_string)
        .collect::<Vec<_>>()
        .join(" ")
}

impl TablePaper {
    /// Convert a paper to its table view counterpart.
    pub fn from_paper(p: Paper, now: chrono::NaiveDateTime) -> Self {
        let age = now - p.created_at;
        let age = match age.to_std() {
            Ok(duration) => duration,
            Err(_) => (-age).to_std().unwrap(),
        };
        Self {
            id: p.id,
            url: p.url,
            filename: p.filename,
            title: p.title,
            tags: p.tags,
            labels: p.labels,
            authors: p.authors,
            age,
        }
    }
}
