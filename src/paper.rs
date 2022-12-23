#[derive(Debug)]
pub struct Paper {
    pub id: i32,
    pub url: Option<String>,
    pub filename: String,
    pub title: Option<String>,
    pub tags: Vec<String>,
}
