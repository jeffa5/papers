use std::collections::BTreeSet;
use std::fs::canonicalize;
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::author::Author;
use crate::db;
use crate::label::Label;
use crate::tag::Tag;
use crate::{db::Db, paper::Paper};

fn now_naive() -> chrono::NaiveDateTime {
    let n = chrono::Utc::now().naive_utc();
    let millis = n.timestamp();
    chrono::NaiveDateTime::from_timestamp_opt(millis, 0).unwrap()
}

pub struct Repo {
    db: Db,
    root: PathBuf,
}

impl Repo {
    #[cfg(test)]
    pub fn in_memory(root: &Path) -> anyhow::Result<Self> {
        let db = Db::in_memory()?;
        Ok(Self {
            db,
            root: canonicalize(root)?,
        })
    }

    pub fn init(root: &Path, db_file: &Path) -> anyhow::Result<Self> {
        let db = Db::init(root, db_file)?;
        Ok(Self {
            db,
            root: canonicalize(root)?,
        })
    }

    pub fn load(root: &Path, db_file: &Path) -> anyhow::Result<Self> {
        let db = Db::load(root, db_file)?;
        Ok(Self {
            db,
            root: canonicalize(root)?,
        })
    }

    pub fn add<P: AsRef<Path>>(
        &mut self,
        file: Option<P>,
        url: Option<String>,
        title: Option<String>,
        authors: BTreeSet<Author>,
        tags: BTreeSet<Tag>,
        labels: BTreeSet<Label>,
    ) -> anyhow::Result<Paper> {
        let filename = if let Some(file) = file {
            let file = file.as_ref();
            if !canonicalize(file)
                .context("canonicalising the filename")?
                .parent()
                .unwrap()
                .starts_with(&self.root)
            {
                anyhow::bail!(
                    "File doesn't live in the root {}",
                    self.root.to_string_lossy()
                )
            }

            let file = file.file_name().unwrap();
            Some(file.to_string_lossy().into_owned())
        } else {
            None
        };

        let paper = db::NewPaper {
            url,
            filename,
            title,
            created_at: None,
            modified_at: now_naive(),
        };
        let paper = self.db.insert_paper(paper)?;

        let new_authors = authors
            .iter()
            .map(|t| db::NewAuthor {
                paper_id: paper.id,
                author: t.to_string(),
            })
            .collect();
        self.db.insert_authors(new_authors)?;

        let new_tags = tags
            .iter()
            .map(|t| db::NewTag {
                paper_id: paper.id,
                tag: t.to_string(),
            })
            .collect();
        self.db.insert_tags(new_tags)?;

        let new_labels = labels
            .iter()
            .map(|l| db::NewLabel {
                paper_id: paper.id,
                label_key: l.key().to_owned(),
                label_value: l.value().to_owned(),
            })
            .collect();
        self.db.insert_labels(new_labels)?;

        Ok(Paper {
            id: paper.id,
            url: paper.url,
            filename: paper.filename,
            title: paper.title,
            authors,
            tags,
            labels,
            notes: None,
            deleted: paper.deleted,
            created_at: paper.created_at,
            modified_at: paper.modified_at,
        })
    }

    pub fn import(&mut self, paper: Paper) -> anyhow::Result<i32> {
        let db_paper = db::NewPaper {
            url: paper.url,
            filename: paper.filename,
            title: paper.title,
            created_at: Some(paper.created_at),
            modified_at: paper.modified_at,
        };
        let db_paper = self.db.insert_paper(db_paper)?;

        let new_authors = paper
            .authors
            .iter()
            .map(|t| db::NewAuthor {
                paper_id: db_paper.id,
                author: t.to_string(),
            })
            .collect();
        self.db.insert_authors(new_authors)?;

        let new_tags = paper
            .tags
            .iter()
            .map(|t| db::NewTag {
                paper_id: db_paper.id,
                tag: t.to_string(),
            })
            .collect();
        self.db.insert_tags(new_tags)?;

        let new_labels = paper
            .labels
            .iter()
            .map(|l| db::NewLabel {
                paper_id: db_paper.id,
                label_key: l.key().to_owned(),
                label_value: l.value().to_owned(),
            })
            .collect();
        self.db.insert_labels(new_labels)?;

        if let Some(notes) = paper.notes {
            let note = db::NewNote {
                paper_id: db_paper.id,
                content: notes,
            };
            self.db.insert_note(note)?;
        }

        // delete it if it was in the old version.
        if paper.deleted{
            self.db.remove_paper(db_paper.id)?;
        }

        Ok(db_paper.id)
    }

    pub fn update<P: AsRef<Path>>(
        &mut self,
        paper_id: i32,
        file: Option<&P>,
        url: Option<Option<String>>,
        title: Option<Option<String>>,
    ) -> anyhow::Result<()> {
        let filename = if let Some(file) = file {
            let file = file.as_ref();
            if !canonicalize(file)?
                .parent()
                .unwrap()
                .starts_with(&self.root)
            {
                anyhow::bail!(
                    "File doesn't live in the root {}",
                    self.root.to_string_lossy()
                )
            }

            Some(file.file_name().unwrap().to_string_lossy().into_owned())
        } else {
            None
        };

        let paper_update = db::PaperUpdate {
            id: paper_id,
            url,
            filename,
            title,
            modified_at: now_naive(),
        };

        self.db.update_paper(paper_update)?;
        Ok(())
    }

    pub fn remove(&mut self, paper_id: i32) -> anyhow::Result<()> {
        self.db.remove_paper(paper_id)?;
        Ok(())
    }

    pub fn add_authors(&mut self, paper_id: i32, authors: Vec<Author>) -> anyhow::Result<()> {
        let new_authors = authors
            .iter()
            .map(|t| db::NewAuthor {
                paper_id,
                author: t.to_string(),
            })
            .collect();
        self.db.insert_authors(new_authors)?;
        Ok(())
    }

    pub fn remove_authors(&mut self, paper_id: i32, authors: Vec<Author>) -> anyhow::Result<()> {
        let new_authors = authors
            .iter()
            .map(|t| db::NewAuthor {
                paper_id,
                author: t.to_string(),
            })
            .collect();
        self.db.remove_authors(new_authors)?;
        Ok(())
    }

    pub fn add_tags(&mut self, paper_id: i32, tags: Vec<Tag>) -> anyhow::Result<()> {
        let new_tags = tags
            .iter()
            .map(|t| db::NewTag {
                paper_id,
                tag: t.to_string(),
            })
            .collect();
        self.db.insert_tags(new_tags)?;
        Ok(())
    }

    pub fn remove_tags(&mut self, paper_id: i32, tags: Vec<Tag>) -> anyhow::Result<()> {
        let new_tags = tags
            .iter()
            .map(|t| db::NewTag {
                paper_id,
                tag: t.to_string(),
            })
            .collect();
        self.db.remove_tags(new_tags)?;
        Ok(())
    }

    pub fn add_labels(&mut self, paper_id: i32, labels: Vec<Label>) -> anyhow::Result<()> {
        let new_labels = labels
            .iter()
            .map(|l| db::NewLabel {
                paper_id,
                label_key: l.key().to_owned(),
                label_value: l.value().to_owned(),
            })
            .collect();
        self.db.insert_labels(new_labels)?;
        Ok(())
    }

    pub fn remove_labels(&mut self, paper_id: i32, labels: Vec<Tag>) -> anyhow::Result<()> {
        let new_labels = labels
            .iter()
            .map(|t| db::DeleteLabel {
                paper_id,
                label_key: t.key().to_owned(),
            })
            .collect();
        self.db.remove_labels(new_labels)?;
        Ok(())
    }
    pub fn get_paper(&mut self, paper_id: i32) -> anyhow::Result<Paper> {
        let db_paper = self.db.get_paper(paper_id)?;
        if db_paper.deleted {
            anyhow::bail!("Paper not found");
        }

        let authors: BTreeSet<_> = self
            .db
            .get_authors(paper_id)?
            .into_iter()
            .map(|a| Author::new(&a.author))
            .collect();

        let tags: BTreeSet<_> = self
            .db
            .get_tags(paper_id)?
            .into_iter()
            .map(|t| Tag::new(&t.tag))
            .collect();

        let labels: BTreeSet<_> = self
            .db
            .get_labels(paper_id)?
            .into_iter()
            .map(|l| Label::new(&l.label_key, &l.label_value))
            .collect();

        let notes = self.db.get_note(paper_id).map(|n| n.content).ok();

        Ok(Paper {
            id: paper_id,
            url: db_paper.url,
            filename: db_paper.filename,
            title: db_paper.title,
            authors,
            tags,
            labels,
            notes,
            deleted: db_paper.deleted,
            created_at: db_paper.created_at,
            modified_at: db_paper.modified_at,
        })
    }

    pub fn list(
        &mut self,
        match_ids: Vec<i32>,
        match_file: Option<String>,
        match_title: Option<String>,
        match_authors: Vec<Author>,
        match_tags: Vec<Tag>,
        match_labels: Vec<Label>,
    ) -> anyhow::Result<Vec<Paper>> {
        let db_papers = self.db.list_papers()?;
        let mut papers = Vec::new();
        let match_title = match_title.map(|t| t.to_lowercase());
        let match_file = match_file.map(|t| t.to_lowercase());
        for paper in db_papers {
            if paper.deleted {
                continue;
            }

            if !match_ids.is_empty() && !match_ids.contains(&paper.id) {
                continue;
            }

            let authors: BTreeSet<_> = self
                .db
                .get_authors(paper.id)?
                .into_iter()
                .map(|a| Author::new(&a.author))
                .collect();

            let tags: BTreeSet<_> = self
                .db
                .get_tags(paper.id)?
                .into_iter()
                .map(|t| Tag::new(&t.tag))
                .collect();

            let labels: BTreeSet<_> = self
                .db
                .get_labels(paper.id)?
                .into_iter()
                .map(|l| Label::new(&l.label_key, &l.label_value))
                .collect();

            let notes = self.db.get_note(paper.id).map(|n| n.content).ok();

            if let Some(match_file) = match_file.as_ref() {
                if let Some(filename) = paper.filename.as_ref() {
                    if !filename.to_lowercase().contains(match_file) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

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
            if !match_authors.iter().all(|a| authors.contains(a)) {
                continue;
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
                authors,
                tags,
                labels,
                notes,
                deleted: paper.deleted,
                created_at: paper.created_at,
                modified_at: paper.modified_at,
            });
        }
        Ok(papers)
    }

    pub fn get_note(&mut self, paper_id: i32) -> anyhow::Result<db::Note> {
        if let Ok(note) = self.db.get_note(paper_id) {
            return Ok(note);
        }
        let note = db::NewNote {
            paper_id,
            content: String::new(),
        };
        self.db.insert_note(note)?;
        self.db.get_note(paper_id)
    }

    pub fn update_note(&mut self, note: db::Note) -> anyhow::Result<()> {
        self.db.update_note(note)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use chrono::NaiveDateTime;
    use expect_test::expect;

    use super::*;

    #[test]
    fn test_create_paper() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut repo = Repo::in_memory(dir.path()).unwrap();
        let path = dir.path().join("file");
        File::create(&path).unwrap();
        let created = NaiveDateTime::default();
        let mut paper = repo
            .add(
                Some(&path),
                Some("blah".to_owned()),
                Some("title".to_owned()),
                BTreeSet::from_iter(vec![Author::new("a"), Author::new("b")]),
                BTreeSet::from_iter(vec![Tag::new("t1"), Tag::new("t2")]),
                BTreeSet::from_iter(vec![Label::new("k", "v")]),
            )
            .unwrap();
        paper.created_at = created;
        paper.modified_at = created;
        let expect = expect![[r#"
            Paper {
                id: 1,
                url: Some(
                    "blah",
                ),
                filename: Some(
                    "file",
                ),
                title: Some(
                    "title",
                ),
                tags: {
                    Tag {
                        key: "t1",
                    },
                    Tag {
                        key: "t2",
                    },
                },
                labels: {
                    Label {
                        key: "k",
                        value: "v",
                    },
                },
                authors: {
                    Author {
                        author: "a",
                    },
                    Author {
                        author: "b",
                    },
                },
                notes: None,
                deleted: false,
                created_at: 1970-01-01T00:00:00,
                modified_at: 1970-01-01T00:00:00,
            }
        "#]];
        expect.assert_debug_eq(&paper);
    }

    #[test]
    fn test_create_remove_paper() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut repo = Repo::in_memory(dir.path()).unwrap();
        let path = dir.path().join("file");
        File::create(&path).unwrap();
        let paper = repo
            .add(
                Some(&path),
                Some("blah".to_owned()),
                Some("title".to_owned()),
                BTreeSet::from_iter(vec![Author::new("a"), Author::new("b")]),
                BTreeSet::from_iter(vec![Tag::new("t1"), Tag::new("t2")]),
                BTreeSet::from_iter(vec![Label::new("k", "v")]),
            )
            .unwrap();
        repo.remove(paper.id).unwrap();
        let paper = repo.get_paper(paper.id);

        let expect = expect![[r#"
            Err(
                "Paper not found",
            )
        "#]];
        expect.assert_debug_eq(&paper);
    }
}
