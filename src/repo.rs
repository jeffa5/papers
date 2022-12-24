use std::fs::canonicalize;
use std::path::{Path, PathBuf};

use crate::db;
use crate::label::Label;
use crate::tag::Tag;
use crate::{db::Db, paper::Paper};

pub struct Repo {
    db: Db,
    root: PathBuf,
}

impl Repo {
    #[must_use]
    pub fn init(root: &Path) -> Self {
        let db = Db::init(root);
        Self {
            db,
            root: canonicalize(root).unwrap(),
        }
    }

    #[must_use]
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
        tags: Vec<Tag>,
        labels: Vec<Label>,
    ) -> Paper {
        let file = file.as_ref();
        if !canonicalize(file)
            .unwrap()
            .parent()
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
        let new_tags = tags
            .iter()
            .map(|t| db::NewTag {
                paper_id: paper.id,
                tag: t.to_string(),
            })
            .collect();
        self.db.insert_tags(new_tags);

        let new_labels = labels
            .iter()
            .map(|l| db::NewLabel {
                paper_id: paper.id,
                label_key: l.key().to_owned(),
                label_value: l.value().to_owned(),
            })
            .collect();
        self.db.insert_labels(new_labels);

        Paper {
            id: paper.id,
            url: paper.url,
            filename: paper.filename,
            title: paper.title,
            tags,
            labels,
            notes: false,
        }
    }

    pub fn update<P: AsRef<Path>>(
        &mut self,
        paper_id: i32,
        file: Option<&P>,
        url: Option<Option<String>>,
        title: Option<Option<String>>,
    ) {
        let filename = file.map(|file| {
            let file = file.as_ref();
            if !canonicalize(file)
                .unwrap()
                .parent()
                .unwrap()
                .starts_with(&self.root)
            {
                panic!("file doesn't live in the root")
            }

            file.file_name().unwrap().to_string_lossy().into_owned()
        });

        let paper_update = db::PaperUpdate {
            id: paper_id,
            url,
            filename,
            title,
        };

        self.db.update_paper(paper_update);
    }

    pub fn list(
        &mut self,
        match_title: Option<String>,
        match_tags: Vec<Tag>,
        match_labels: Vec<Label>,
    ) -> Vec<Paper> {
        let db_papers = self.db.list_papers();
        let mut papers = Vec::new();
        let match_title = match_title.map(|t| t.to_lowercase());
        for paper in db_papers {
            let tags: Vec<_> = self
                .db
                .get_tags(paper.id)
                .into_iter()
                .map(|t| Tag::new(&t.tag))
                .collect();

            let labels: Vec<_> = self
                .db
                .get_labels(paper.id)
                .into_iter()
                .map(|l| Label::new(&l.label_key, &l.label_value))
                .collect();

            let notes = self.db.get_note(paper.id).is_some();

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
                notes,
            });
        }
        papers
    }

    pub fn get_note(&mut self, paper_id: i32) -> db::Note {
        if let Some(note) = self.db.get_note(paper_id) {
            return note;
        }
        let note = db::NewNote {
            paper_id,
            content: String::new(),
        };
        self.db.insert_note(note);
        self.db.get_note(paper_id).unwrap()
    }

    pub fn update_note(&mut self, note: db::Note) {
        self.db.update_note(note)
    }
}
