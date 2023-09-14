use std::collections::BTreeSet;

use crate::{author::Author, label::Label, tag::Tag};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
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

impl Paper {
    pub fn into_editable_and_read_only(self) -> (EditablePaperData, ReadOnlyPaperData) {
        let Paper {
            id,
            url,
            filename,
            title,
            tags,
            labels,
            authors,
            notes,
            deleted,
            created_at,
            modified_at,
        } = self;
        (
            EditablePaperData {
                url,
                filename,
                title,
                tags,
                labels,
                authors,
            },
            ReadOnlyPaperData {
                id,
                notes,
                deleted,
                created_at,
                modified_at,
            },
        )
    }

    pub fn into_export_data(self) -> ExportPaperData {
        let Paper {
            id: _,
            url,
            filename,
            title,
            tags,
            labels,
            authors,
            notes: _,
            deleted: _,
            created_at,
            modified_at,
        } = self;
        ExportPaperData {
            url,
            filename,
            title: title.unwrap_or_default(),
            tags,
            labels,
            authors,
            created_at,
            modified_at,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EditablePaperData {
    pub url: Option<String>,
    pub filename: Option<String>,
    pub title: Option<String>,
    pub tags: BTreeSet<Tag>,
    pub labels: BTreeSet<Label>,
    pub authors: BTreeSet<Author>,
}

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

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ReadOnlyPaperData {
    pub id: i32,
    pub notes: Option<String>,
    pub deleted: bool,
    pub created_at: chrono::NaiveDateTime,
    pub modified_at: chrono::NaiveDateTime,
}
