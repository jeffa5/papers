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

    pub fn add<P: AsRef<Path>>(
        &mut self,
        file: &P,
        url: Option<String>,
        title: Option<String>,
        tags: Vec<String>,
    ) {
        let paper = db::NewPaper {
            url,
            filename: file.as_ref().to_string_lossy().into_owned(),
            title,
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

    pub fn list(&mut self, match_title: Option<String>, match_tags: Vec<String>) -> Vec<Paper> {
        let db_papers = self.db.list_papers();
        let mut papers = Vec::new();
        let match_title = match_title.map(|t| t.to_lowercase());
        for paper in db_papers {
            let tags: Vec<String> = self
                .db
                .get_tags(paper.id)
                .into_iter()
                .map(|t| t.tag)
                .collect();

            if let Some(match_title) = match_title.as_ref() {
                if let Some(title) = paper.title.as_ref() {
                    if !title.to_lowercase().contains(match_title) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            // TODO: push this into the DB layer
            // filter papers down
            if !match_tags.iter().all(|t| tags.contains(t)) {
                continue;
            }

            papers.push(Paper {
                id: paper.id,
                url: paper.url,
                filename: paper.filename,
                title: paper.title,
                tags,
            });
        }
        papers
    }
}
