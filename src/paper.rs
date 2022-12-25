use crate::{author::Author, label::Label, tag::Tag};
use cli_table::Table;
use serde::Serialize;

#[derive(Debug, Table, Serialize)]
pub struct Paper {
    pub id: i32,
    #[table(display_fn = "display_optional_string")]
    pub url: Option<String>,
    pub filename: String,
    #[table(display_fn = "display_optional_string")]
    pub title: Option<String>,
    #[table(display_fn = "display_tag_vector")]
    pub tags: Vec<Tag>,
    #[table(display_fn = "display_label_vector")]
    pub labels: Vec<Label>,
    #[table(display_fn = "display_author_vector")]
    pub authors: Vec<Author>,
    /// Whether this paper has notes or not
    pub notes: bool,
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
