use std::{collections::BTreeSet, fmt::Display, time::Duration};

use comfy_table::Cell;
use papers_core::{author::Author, label::Label, paper::Paper, tag::Tag};
use serde::Serialize;

/// Paper format for display in a table.
#[derive(Debug, Serialize)]
pub struct TablePaper {
    /// Id of the paper.
    pub id: i32,
    /// Url the paper was fetched from.
    pub url: Option<String>,
    /// Local filename of the document.
    pub filename: Option<String>,
    /// Title of the document.
    pub title: Option<String>,
    /// Tags for this document.
    pub tags: BTreeSet<Tag>,
    /// Labels for this document.
    pub labels: BTreeSet<Label>,
    /// Authors for this document.
    pub authors: BTreeSet<Author>,
    /// Age since creation.
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

    fn to_row(&self) -> comfy_table::Row {
        let id = self.id.to_string();
        let title = self.title.iter().next().cloned().unwrap_or_default();
        let tags = self
            .tags
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let labels = self
            .labels
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let authors = self
            .authors
            .iter()
            .map(|t| t.to_string())
            .collect::<Vec<_>>()
            .join(", ");
        let age = display_duration(&self.age);

        let columns = vec![id, title, authors, tags, labels, age];

        let mut row = comfy_table::Row::from(columns);
        row.max_height(1);
        row
    }
}

/// A way to print tables to the terminal.
pub struct Table {
    papers: Vec<TablePaper>,
}

fn now_naive() -> chrono::NaiveDateTime {
    let n = chrono::Utc::now().naive_utc();
    let millis = n.timestamp();
    chrono::NaiveDateTime::from_timestamp_opt(millis, 0).unwrap()
}

impl From<Vec<Paper>> for Table {
    fn from(v: Vec<Paper>) -> Self {
        let now = now_naive();
        let papers = v
            .into_iter()
            .map(|p| TablePaper::from_paper(p, now))
            .collect();
        Self { papers }
    }
}

impl Table {
    fn header() -> comfy_table::Row {
        comfy_table::Row::from(vec!["id", "title", "authors", "tags", "labels", "age"])
    }
}

impl Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tab = comfy_table::Table::new();
        tab.load_preset(comfy_table::presets::UTF8_FULL_CONDENSED)
            .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
            .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

        tab.set_header(Self::header());

        let authors_column = tab.column_mut(2).unwrap();
        authors_column.set_delimiter(',');

        for paper in &self.papers {
            tab.add_row(paper.to_row());
        }

        write!(f, "{}", tab)
    }
}
