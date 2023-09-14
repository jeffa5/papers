use gray_matter::{engine::YAML, Matter};
use std::collections::BTreeSet;
use std::fs::{canonicalize, read_dir, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::author::Author;
use crate::label::Label;
use crate::paper::Paper;
use crate::tag::Tag;

pub const PROHIBITED_PATH_CHARS: &[char] =
    &['/', '\\', '?', '%', '*', ':', '|', '"', '<', '>', '.'];

fn now_naive() -> chrono::NaiveDateTime {
    let n = chrono::Utc::now().naive_utc();
    let millis = n.timestamp();
    chrono::NaiveDateTime::from_timestamp_opt(millis, 0).unwrap()
}

pub struct Repo {
    root: PathBuf,
}

impl Repo {
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn load(root: &Path) -> anyhow::Result<Self> {
        Ok(Self {
            root: canonicalize(root)?,
        })
    }

    pub fn add<P: AsRef<Path>>(
        &mut self,
        file: Option<P>,
        url: Option<String>,
        title: String,
        authors: BTreeSet<Author>,
        tags: BTreeSet<Tag>,
        labels: BTreeSet<Label>,
    ) -> anyhow::Result<Paper> {
        let filename = if let Some(file) = file {
            let file = file.as_ref();
            let file = canonicalize(file).context("canonicalising the filename")?;
            let file = file
                .strip_prefix(&self.root)
                .context("File does not live in the root")?;
            Some(file.to_string_lossy().into_owned())
        } else {
            None
        };
        let paper = Paper {
            title,
            url,
            filename,
            tags,
            labels,
            authors,
            created_at: now_naive(),
            modified_at: now_naive(),
        };
        let paper_path = self.get_path(&paper);
        if self.root.join(&paper_path).is_file() {
            anyhow::bail!("Paper entry already exists for {:?}", paper_path);
        }
        self.write_paper(&paper, "")?;

        Ok(paper)
    }

    pub fn import(&mut self, paper: Paper) -> anyhow::Result<()> {
        self.write_paper(&paper, "")
    }

    pub fn write_paper(&self, paper: &Paper, notes: &str) -> anyhow::Result<()> {
        let data_string = serde_yaml::to_string(&paper)?;

        let path = self.get_path(paper);
        let path = self.root.join(path);
        let mut file = File::create(path)?;
        write!(file, "---\n{data_string}\n---\n{notes}")?;
        Ok(())
    }

    pub fn update(&mut self, paper: &Paper, file: Option<&Path>) -> anyhow::Result<()> {
        let filename = if let Some(file) = file {
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

        let paper_path = self.get_path(paper);
        let (mut data, notes) = self.get_paper_with_notes(&paper_path)?;
        data.filename = filename;

        self.write_paper(&data, &notes)?;

        Ok(())
    }

    pub fn list(
        &mut self,
        match_file: Option<String>,
        match_title: Option<String>,
        match_authors: Vec<Author>,
        match_tags: Vec<Tag>,
        match_labels: Vec<Label>,
    ) -> anyhow::Result<Vec<Paper>> {
        let papers = self.all_papers();
        let mut filtered_papers = Vec::new();
        let match_title = match_title.map(|t| t.to_lowercase());
        let match_file = match_file.map(|t| t.to_lowercase());
        for paper in papers {
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
                if !paper.title.to_lowercase().contains(match_title) {
                    continue;
                }
            }

            // filter papers down
            if !match_authors.iter().all(|a| paper.authors.contains(a)) {
                continue;
            }

            // TODO: push this into the DB layer
            // filter papers down
            if !match_tags.iter().all(|t| paper.tags.contains(t)) {
                continue;
            }

            // TODO: push this into the DB layer
            // filter papers down
            if !match_labels.iter().all(|l| paper.labels.contains(l)) {
                continue;
            }

            filtered_papers.push(paper);
        }
        Ok(filtered_papers)
    }

    pub fn get_path(&self, paper: &Paper) -> PathBuf {
        let title = paper.title.replace(PROHIBITED_PATH_CHARS, "");
        PathBuf::from(&title).with_extension("md")
    }

    pub fn all_papers(&self) -> Vec<Paper> {
        let mut papers = Vec::new();
        let entries = read_dir(&self.root);
        if let Ok(entries) = entries {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.extension().and_then(|e| e.to_str()) == Some("md") {
                        if let Ok(paper) = self.get_paper(&path) {
                            papers.push(paper);
                        }
                    }
                }
            }
        }
        papers
    }

    pub fn get_paper(&self, path: &Path) -> anyhow::Result<Paper> {
        self.get_paper_with_notes(path).map(|(d, _)| d)
    }

    pub fn get_paper_with_notes(&self, path: &Path) -> anyhow::Result<(Paper, String)> {
        let mut file_content = String::new();
        let path = if path.is_absolute() {
            path.to_owned()
        } else {
            self.root.join(path)
        };
        let mut file = File::open(path)?;
        file.read_to_string(&mut file_content)?;
        let matter = Matter::<YAML>::new();
        let file_content = matter.parse(&file_content);
        if let Some(data) = file_content.data {
            let paper = data.deserialize::<Paper>()?;
            Ok((paper, file_content.content))
        } else {
            anyhow::bail!("No content for file! Is there any frontmatter?")
        }
    }
}
