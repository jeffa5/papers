use std::path::Path;

use crate::db;
use crate::{db::Db, paper::Paper};

pub struct Repo {
    db: Db,
}

impl Repo {
    pub fn init(dir: &Path) -> Self {
        let db = Db::init(dir);
        Self { db }
    }

    pub fn load(dir: &Path) -> Self {
        let db = Db::load(dir);
        Self { db }
    }

    pub fn add(&mut self, file: &Path, tags: Vec<String>) {
        let paper = db::NewPaper {
            url: None,
            filename: file.to_string_lossy().into_owned(),
        };
        let paper = self.db.insert_paper(paper);
        let tags = tags
            .into_iter()
            .map(|t| db::NewTag {
                paper_id: paper.id,
                tag: t,
            })
            .collect();
        self.db.insert_tags(tags);
    }

    pub fn list(&mut self) -> Vec<Paper> {
        let db_papers = self.db.list_papers();
        let mut papers = Vec::new();
        for paper in db_papers {
            let tags = self
                .db
                .get_tags(paper.id)
                .into_iter()
                .map(|t| t.tag)
                .collect();
            papers.push(Paper {
                id: paper.id,
                url: paper.url,
                filename: paper.filename,
                tags,
            });
        }
        papers
    }
}
