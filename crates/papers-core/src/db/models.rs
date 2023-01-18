use super::schema::authors;
use super::schema::labels;
use super::schema::notes;
use super::schema::papers;
use super::schema::tags;
use diesel::prelude::*;

#[derive(Debug, Queryable)]
pub struct Paper {
    pub id: i32,
    pub url: Option<String>,
    pub filename: Option<String>,
    pub title: Option<String>,
    pub deleted: bool,
    pub created_at: chrono::NaiveDateTime,
    pub modified_at: chrono::NaiveDateTime,
}

#[derive(Insertable)]
#[diesel(table_name = papers)]
pub struct NewPaper {
    pub url: Option<String>,
    pub filename: Option<String>,
    pub title: Option<String>,
    pub modified_at: chrono::NaiveDateTime,
}

#[derive(Debug, Identifiable, AsChangeset)]
#[diesel(table_name = papers)]
pub struct PaperUpdate {
    pub id: i32,
    pub url: Option<Option<String>>,
    pub filename: Option<String>,
    pub title: Option<Option<String>>,
    pub modified_at: chrono::NaiveDateTime,
}

#[derive(Debug, Queryable)]
pub struct Tag {
    pub paper_id: i32,
    pub tag: String,
}

#[derive(Insertable)]
#[diesel(table_name = tags)]
pub struct NewTag {
    pub paper_id: i32,
    pub tag: String,
}

#[derive(Debug, Queryable)]
pub struct Label {
    pub paper_id: i32,
    pub label_key: String,
    pub label_value: String,
}

#[derive(Insertable)]
#[diesel(table_name = labels)]
pub struct NewLabel {
    pub paper_id: i32,
    pub label_key: String,
    pub label_value: String,
}

pub struct DeleteLabel {
    pub paper_id: i32,
    pub label_key: String,
}

#[derive(Debug, Queryable)]
pub struct Note {
    pub id: i32,
    pub paper_id: i32,
    pub content: String,
}

#[derive(Insertable)]
#[diesel(table_name = notes)]
pub struct NewNote {
    pub paper_id: i32,
    pub content: String,
}

#[derive(Debug, Queryable)]
pub struct Author {
    pub paper_id: i32,
    pub author: String,
}

#[derive(Insertable)]
#[diesel(table_name = authors)]
pub struct NewAuthor {
    pub paper_id: i32,
    pub author: String,
}
