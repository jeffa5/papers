use papers_core::{paper::PaperMeta, repo::PROHIBITED_PATH_CHARS};

/// Strategy to rename files.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Strategy {
    /// Rename to match the title of the paper.
    Title,
}

impl Strategy {
    /// Rename a file using the current strategy.
    pub fn rename(&self, paper: &PaperMeta) -> anyhow::Result<String> {
        let name = match self {
            Self::Title => Ok(paper.title.to_owned()),
        };

        name.map(|n| n.replace(PROHIBITED_PATH_CHARS, ""))
    }
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};

    use super::*;

    fn check(strategy: Strategy, paper: PaperMeta, expected: Expect) {
        let renamed = strategy.rename(&paper).unwrap();
        expected.assert_eq(&renamed);
    }

    #[test]
    fn test_with_spaces() {
        check(
            Strategy::Title,
            PaperMeta {
                title: "my long title with spaces".to_owned(),
                ..Default::default()
            },
            expect!["my long title with spaces"],
        );
    }

    #[test]
    fn test_strips_unc() {
        // some chars shouldn't be allowed https://stackoverflow.com/a/4040068/7164655
        check(
            Strategy::Title,
            PaperMeta {
                title: "MLT: my |long<> title\" with/ spaces and * more?".to_owned(),
                ..Default::default()
            },
            expect!["MLT my long title with spaces and  more"],
        );
    }
}
