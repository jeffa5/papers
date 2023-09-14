use std::collections::BTreeSet;

use crate::{author::Author, label::Label, tag::Tag};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ExportPaperData {
    pub title: String,
    pub url: Option<String>,
    pub filename: Option<String>,
    pub tags: BTreeSet<Tag>,
    pub labels: BTreeSet<Label>,
    pub authors: BTreeSet<Author>,
    pub created_at: chrono::NaiveDateTime,
    pub modified_at: chrono::NaiveDateTime,
}
