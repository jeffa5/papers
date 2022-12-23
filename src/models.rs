use crate::schema::papers;
use diesel::prelude::*;

#[derive(Debug, Queryable)]
pub struct Paper {
    pub id: i32,
    pub url: Option<String>,
    pub filename: String,
}

#[derive(Insertable)]
#[diesel(table_name = papers)]
pub struct NewPaper {
    pub url: Option<String>,
    pub filename: String,
}