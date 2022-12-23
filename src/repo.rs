use std::{ path::Path};

use crate::{
    db::Db,
    models::{NewPaper, Paper},
};

pub struct Repo {
    db: Db,
}

impl Repo {
    pub fn init(dir: &Path) -> Self {
        let db = Db::init(dir);
        Self { db }
    }

    pub fn load(dir:&Path) -> Self {
        let db = Db::load(dir);
        Self { db }
    }

    pub fn add(&mut self, file: &Path) {
        let paper = NewPaper {
            url: None,
            filename: file.to_string_lossy().into_owned(),
        };
        self.db.insert_paper(paper);
    }

    pub fn list(&mut self) -> Vec<Paper> {
        self.db.list()
    }
}
