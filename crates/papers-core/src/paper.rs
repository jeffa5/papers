use std::collections::BTreeSet;

use crate::{author::Author, label::Label, tag::Tag};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paper {
    pub id: i32,
    pub url: Option<String>,
    pub filename: Option<String>,
    pub title: Option<String>,
    pub tags: BTreeSet<Tag>,
    pub labels: BTreeSet<Label>,
    pub authors: BTreeSet<Author>,
    pub notes: Option<String>,
    pub deleted: bool,
    pub created_at: chrono::NaiveDateTime,
    pub modified_at: chrono::NaiveDateTime,
}
