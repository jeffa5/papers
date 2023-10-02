use gray_matter::{engine::YAML, Matter};
use std::collections::BTreeSet;
use std::fs::{canonicalize, read_dir, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use anyhow::Context;

use crate::author::Author;
use crate::label::Label;
use crate::paper::{LoadedPaper, PaperMeta};
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
    ) -> anyhow::Result<PaperMeta> {
        let filename = if let Some(file) = file {
            let file = file.as_ref();
            let file = canonicalize(file).context("canonicalising the filename")?;
            let file = file
                .strip_prefix(&self.root)
                .context("File does not live in the root")?;
            Some(file.to_owned())
        } else {
            None
        };
        let paper = PaperMeta {
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
        let paper_path = self.root.join(&paper_path);

        if paper_path.is_file() {
            anyhow::bail!("Paper entry already exists for {:?}", paper_path);
        }
        self.write_paper(&paper_path, paper.clone(), "")?;

        Ok(paper)
    }

    pub fn import(&mut self, paper: PaperMeta) -> anyhow::Result<()> {
        let paper_path = self.get_path(&paper);
        self.write_paper(&paper_path, paper, "")
    }

    pub fn write_paper(&self, path: &Path, mut paper: PaperMeta, notes: &str) -> anyhow::Result<()> {
        paper.modified_at = now_naive();
        let data_string = serde_yaml::to_string(&paper)?;

        let path = self.root.join(path);
        let mut file = File::create(path)?;
        write!(file, "---\n{data_string}---\n{notes}")?;
        Ok(())
    }

    pub fn update(&self, paper: &LoadedPaper, file: Option<&Path>) -> anyhow::Result<()> {
        let filename = if let Some(file) = file {
            if !canonicalize(file)
                .with_context(|| format!("Canoncalizing file path {:?}", file))?
                .parent()
                .unwrap()
                .starts_with(&self.root)
            {
                anyhow::bail!("File doesn't live in the root {:?}", self.root)
            }

            Some(file.file_name().unwrap_or_default().into())
        } else {
            None
        };

        let mut paper = self
            .get_paper(&paper.path)
            .with_context(|| format!("Opening paper notes at {:?}", paper.path))?;
        paper.meta.filename = filename;

        self.write_paper(&paper.path, paper.meta, &paper.notes)
            .with_context(|| format!("Writing paper {:?}", paper.path))?;

        Ok(())
    }

    pub fn list(
        &mut self,
        match_file: Option<String>,
        match_title: Option<String>,
        match_authors: Vec<Author>,
        match_tags: Vec<Tag>,
        match_labels: Vec<Label>,
    ) -> anyhow::Result<Vec<LoadedPaper>> {
        let papers = self.all_papers();
        let mut filtered_papers = Vec::new();
        let match_title = match_title.map(|t| t.to_lowercase());
        let match_file = match_file.map(|t| t.to_lowercase());
        for paper in papers {
            if let Some(match_file) = match_file.as_ref() {
                if let Some(filename) = paper.meta.filename.as_ref() {
                    let filename_str = filename.to_string_lossy().into_owned();
                    if !filename_str.to_lowercase().contains(match_file) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            if let Some(match_title) = match_title.as_ref() {
                if !paper.meta.title.to_lowercase().contains(match_title) {
                    continue;
                }
            }

            // filter papers down
            if !match_authors.iter().all(|a| paper.meta.authors.contains(a)) {
                continue;
            }

            // filter papers down
            if !match_tags.iter().all(|t| paper.meta.tags.contains(t)) {
                continue;
            }

            // filter papers down
            if !match_labels.iter().all(|l| paper.meta.labels.contains(l)) {
                continue;
            }

            filtered_papers.push(paper);
        }
        Ok(filtered_papers)
    }

    pub fn get_path(&self, paper: &PaperMeta) -> PathBuf {
        let title = paper.title.replace(PROHIBITED_PATH_CHARS, "");
        PathBuf::from(&title).with_extension("md")
    }

    pub fn all_papers(&self) -> Vec<LoadedPaper> {
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

    pub fn get_paper(&self, path: &Path) -> anyhow::Result<LoadedPaper> {
        let mut file_content = String::new();
        let path = if path.is_absolute() {
            path.to_owned()
        } else {
            self.root.join(path)
        };
        let mut file = File::open(&path)?;
        file.read_to_string(&mut file_content)?;
        let matter = Matter::<YAML>::new();
        let file_content = matter.parse(&file_content);
        if let Some(data) = file_content.data {
            let paper = data.deserialize::<PaperMeta>()?;
            let path = path.strip_prefix(&self.root).unwrap().to_owned();
            let notes = file_content.content;
            Ok(LoadedPaper {
                path,
                meta: paper,
                notes,
            })
        } else {
            anyhow::bail!("No content for file! Is there any frontmatter?")
        }
    }
}
