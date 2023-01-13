use std::fs::canonicalize;
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::author::Author;
use crate::db;
use crate::label::Label;
use crate::tag::Tag;
use crate::{db::Db, paper::Paper};

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

    pub fn init(root: &Path) -> anyhow::Result<Self> {
        let db = Db::init(root)?;
        Ok(Self {
            db,
            root: canonicalize(root)?,
        })
    }

    pub fn load(root: &Path) -> anyhow::Result<Self> {
        let db = Db::load(root)?;
        Ok(Self {
            db,
            root: canonicalize(root)?,
        })
    }

    pub fn add<P: AsRef<Path> + ?Sized>(
        &mut self,
        file: &P,
        url: Option<String>,
        title: Option<String>,
        authors: Vec<Author>,
        tags: Vec<Tag>,
        labels: Vec<Label>,
    ) -> anyhow::Result<Paper> {
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

        let paper = db::NewPaper {
            url,
            filename: file.to_string_lossy().into_owned(),
            title,
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
            notes: false,
        })
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
        if db_paper.deleted{
            anyhow::bail!("Paper not found");
        }

        let authors: Vec<_> = self
            .db
            .get_authors(paper_id)?
            .into_iter()
            .map(|a| Author::new(&a.author))
            .collect();

        let tags: Vec<_> = self
            .db
            .get_tags(paper_id)?
            .into_iter()
            .map(|t| Tag::new(&t.tag))
            .collect();

        let labels: Vec<_> = self
            .db
            .get_labels(paper_id)?
            .into_iter()
            .map(|l| Label::new(&l.label_key, &l.label_value))
            .collect();

        let notes = self.db.get_note(paper_id).is_ok();

        Ok(Paper {
            id: paper_id,
            url: db_paper.url,
            filename: db_paper.filename,
            title: db_paper.title,
            authors,
            tags,
            labels,
            notes,
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

            let authors: Vec<_> = self
                .db
                .get_authors(paper.id)?
                .into_iter()
                .map(|a| Author::new(&a.author))
                .collect();

            let tags: Vec<_> = self
                .db
                .get_tags(paper.id)?
                .into_iter()
                .map(|t| Tag::new(&t.tag))
                .collect();

            let labels: Vec<_> = self
                .db
                .get_labels(paper.id)?
                .into_iter()
                .map(|l| Label::new(&l.label_key, &l.label_value))
                .collect();

            let notes = self.db.get_note(paper.id).is_ok();

            if let Some(match_file) = match_file.as_ref() {
                if !paper.filename.to_lowercase().contains(match_file) {
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

    use expect_test::expect;

    use super::*;

    #[test]
    fn test_create_paper() {
        let dir = tempfile::TempDir::new().unwrap();
        let mut repo = Repo::in_memory(dir.path()).unwrap();
        let path = dir.path().join("file");
        File::create(&path).unwrap();
        let paper = repo
            .add(
                &path,
                Some("blah".to_owned()),
                Some("title".to_owned()),
                vec![Author::new("a"), Author::new("b")],
                vec![Tag::new("t1"), Tag::new("t2")],
                vec![Label::new("k", "v")],
            )
            .unwrap();
        let expect = expect![[r#"
            Paper {
                id: 1,
                url: Some(
                    "blah",
                ),
                filename: "file",
                title: Some(
                    "title",
                ),
                tags: [
                    Tag {
                        key: "t1",
                    },
                    Tag {
                        key: "t2",
                    },
                ],
                labels: [
                    Label {
                        key: "k",
                        value: "v",
                    },
                ],
                authors: [
                    Author {
                        author: "a",
                    },
                    Author {
                        author: "b",
                    },
                ],
                notes: false,
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
                &path,
                Some("blah".to_owned()),
                Some("title".to_owned()),
                vec![Author::new("a"), Author::new("b")],
                vec![Tag::new("t1"), Tag::new("t2")],
                vec![Label::new("k", "v")],
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
