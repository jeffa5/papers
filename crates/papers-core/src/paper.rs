use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use crate::{author::Author, tag::Tag};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub struct LoadedPaper {
    pub path: PathBuf,
    pub meta: PaperMeta,
    pub notes: String,
}

#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaperMeta {
    pub title: String,
    pub url: Option<String>,
    pub filename: Option<PathBuf>,
    pub tags: BTreeSet<Tag>,
    pub labels: BTreeMap<String, String>,
    pub authors: BTreeSet<Author>,
    pub created_at: chrono::NaiveDateTime,
    pub modified_at: chrono::NaiveDateTime,
}
