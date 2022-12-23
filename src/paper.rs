use crate::label::Label;
use cli_table::Table;

#[derive(Debug, Table)]
pub struct Paper {
    pub id: i32,
    #[table(display_fn = "display_optional_string")]
    pub url: Option<String>,
    pub filename: String,
    #[table(display_fn = "display_optional_string")]
    pub title: Option<String>,
    #[table(display_fn = "display_string_vector")]
    pub tags: Vec<String>,
    #[table(display_fn = "display_label_vector")]
    pub labels: Vec<Label>,
}

fn display_optional_string(s: &Option<String>) -> String {
    if let Some(s) = s {
        s.clone()
    } else {
        String::new()
    }
}

fn display_string_vector(v: &[String]) -> String {
    v.join(", ")
}

fn display_label_vector(v: &[Label]) -> String {
    v.iter()
        .map(|l| l.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
