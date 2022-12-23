use super::schema::labels;
use super::schema::papers;
use super::schema::tags;
use diesel::prelude::*;

#[derive(Debug, Queryable)]
pub struct Paper {
    pub id: i32,
    pub url: Option<String>,
    pub filename: String,
    pub title: Option<String>,
}

#[derive(Insertable)]
#[diesel(table_name = papers)]
pub struct NewPaper {
    pub url: Option<String>,
    pub filename: String,
    pub title: Option<String>,
}

#[derive(Debug, Queryable)]
pub struct Tag {
    pub id: i32,
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
    pub id: i32,
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
