use std::{collections::BTreeMap, collections::BTreeSet, fmt::Display, time::Duration};

use papers_core::{author::Author, label::Label, paper::PaperMeta, tag::Tag};
use serde::Serialize;

/// Paper format for display in a table.
#[derive(Debug, Serialize)]
pub struct TablePaper {
    /// Url the paper was fetched from.
    pub url: Option<String>,
    /// Local filename of the document.
    pub filename: Option<String>,
    /// Title of the document.
    pub title: String,
    /// Tags for this document.
    pub tags: BTreeSet<Tag>,
    /// Labels for this document.
    pub labels: BTreeSet<Label>,
    /// Authors for this document.
    pub authors: Vec<Author>,
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
    pub fn from_paper(p: PaperMeta, now: chrono::NaiveDateTime) -> Self {
        let age = now - p.created_at;
        let age = match age.to_std() {
            Ok(duration) => duration,
            Err(_) => (-age).to_std().unwrap(),
        };
        let filename = p.filename.map(|f| f.to_string_lossy().into_owned());
        let labels = p
            .labels
            .into_iter()
            .map(|(k, v)| Label::new(&k, v))
            .collect();
        Self {
            url: p.url,
            filename,
            title: p.title,
            tags: p.tags,
            labels,
            authors: p.authors,
            age,
        }
    }

    fn to_row(&self) -> comfy_table::Row {
        let title = self.title.clone();
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

        let columns = vec![title, authors, tags, labels, age];

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

impl From<Vec<PaperMeta>> for Table {
    fn from(v: Vec<PaperMeta>) -> Self {
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
        comfy_table::Row::from(vec!["title", "authors", "tags", "labels", "age"])
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

/// Store counts for groups.
#[derive(Default, Debug, Serialize)]
pub struct TableCount {
    #[serde(flatten)]
    counts: BTreeMap<String, usize>,
    #[serde(skip)]
    sort_by_count: bool,
}

impl TableCount {
    /// Add a new entry.
    pub fn add(mut self, value: String) -> Self {
        *self.counts.entry(value).or_default() += 1;
        self
    }

    /// Sort entries by count when producing table
    pub fn sort_by_count(&mut self) {
        self.sort_by_count = true;
    }

    fn header() -> comfy_table::Row {
        comfy_table::Row::from(vec!["key", "count"])
    }

    fn rows(&self) -> Vec<comfy_table::Row> {
        let mut items: Vec<_> = self.counts.iter().collect();
        if self.sort_by_count {
            items.sort_by_key(|(_, count)| *count);
        }
        items
            .into_iter()
            .map(|(k, c)| comfy_table::Row::from(vec![k, &c.to_string()]))
            .collect()
    }
}

impl Display for TableCount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut tab = comfy_table::Table::new();

        tab.load_preset(comfy_table::presets::UTF8_FULL_CONDENSED)
            .apply_modifier(comfy_table::modifiers::UTF8_ROUND_CORNERS)
            .set_content_arrangement(comfy_table::ContentArrangement::Dynamic);

        tab.set_header(Self::header());

        for row in self.rows() {
            tab.add_row(row);
        }

        write!(f, "{}", tab)
    }
}
