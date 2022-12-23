use std::fs::canonicalize;
use std::path::{Path, PathBuf};

use crate::db;
use crate::label::Label;
use crate::{db::Db, paper::Paper};

pub struct Repo {
    db: Db,
    root: PathBuf,
}

impl Repo {
    pub fn init(root: &Path) -> Self {
        let db = Db::init(root);
        Self {
            db,
            root: canonicalize(root).unwrap(),
        }
    }

    pub fn load(root: &Path) -> Self {
        let db = Db::load(root);
        Self {
            db,
            root: canonicalize(root).unwrap(),
        }
    }

    pub fn add<P: AsRef<Path>>(
        &mut self,
        file: &P,
        url: Option<String>,
        title: Option<String>,
        tags: Vec<String>,
        labels: Vec<Label>,
    ) {
        let file = file.as_ref();
        if !canonicalize(file).unwrap().parent()
            .unwrap()
            .starts_with(&self.root)
        {
            panic!("file doesn't live in the root")
        }

        let file = file.file_name().unwrap();

        let paper = db::NewPaper {
            url,
            filename: file.to_string_lossy().into_owned(),
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

        let labels = labels
            .into_iter()
            .map(|l| db::NewLabel {
                paper_id: paper.id,
                label_key: l.key().to_owned(),
                label_value: l.value().to_owned(),
            })
            .collect();
        self.db.insert_labels(labels);
    }

    pub fn list(
        &mut self,
        match_title: Option<String>,
        match_tags: Vec<String>,
        match_labels: Vec<Label>,
    ) -> Vec<Paper> {
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

            let labels: Vec<Label> = self
                .db
                .get_labels(paper.id)
                .into_iter()
                .map(|l| Label::new(&l.label_key, &l.label_value))
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

            // TODO: push this into the DB layer
            // filter papers down
            if !match_labels.iter().all(|l| labels.contains(l)) {
                continue;
            }

            papers.push(Paper {
                id: paper.id,
                url: paper.url,
                filename: paper.filename,
                title: paper.title,
                tags,
                labels,
            });
        }
        papers
    }
}
