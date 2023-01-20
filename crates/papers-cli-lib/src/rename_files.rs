use papers_core::paper::Paper;

/// Strategy to rename files.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Strategy {
    /// Rename to match the title of the paper.
    Title,
    /// Rename to match the id of the paper.
    Id,
}

impl Strategy {
    /// Rename a file using the current strategy.
    pub fn rename(&self, paper: &Paper) -> anyhow::Result<String> {
        match self {
            Self::Title => {
                if let Some(title) = &paper.title {
                    Ok(title.to_owned())
                } else {
                    anyhow::bail!("missing title")
                }
            }
            Self::Id => Ok(paper.id.to_string()),
        }
    }
}
