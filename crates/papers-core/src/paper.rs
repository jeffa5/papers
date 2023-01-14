use crate::{author::Author, label::Label, tag::Tag};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Paper {
    pub id: i32,
    pub url: Option<String>,
    pub filename: String,
    pub title: Option<String>,
    pub tags: Vec<Tag>,
    pub labels: Vec<Label>,
    pub authors: Vec<Author>,
    pub notes: Option<String>,
    pub deleted: bool,
    pub created_at: chrono::NaiveDateTime,
    pub modified_at: chrono::NaiveDateTime,
}
