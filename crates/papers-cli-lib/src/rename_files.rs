use papers_core::paper::Paper;

/// Strategy to rename files.
#[derive(Debug, Clone, clap::ValueEnum)]
pub enum Strategy {
    /// Rename to match the title of the paper.
    Title,
    /// Rename to match the id of the paper.
    Id,
}

const PROHIBITED_CHARS: &[char] = &['/', '\\', '?', '%', '*', ':', '|', '"', '<', '>', '.'];

impl Strategy {
    /// Rename a file using the current strategy.
    pub fn rename(&self, paper: &Paper) -> anyhow::Result<String> {
        let name = match self {
            Self::Title => {
                if let Some(title) = &paper.title {
                    Ok(title.to_owned())
                } else {
                    anyhow::bail!("missing title")
                }
            }
            Self::Id => Ok(paper.id.to_string()),
        };

        name.map(|n| n.replace(PROHIBITED_CHARS, ""))
    }
}

#[cfg(test)]
mod tests {
    use expect_test::{expect, Expect};

    use super::*;

    fn check(strategy: Strategy, paper: Paper, expected: Expect) {
        let renamed = strategy.rename(&paper).unwrap();
        expected.assert_eq(&renamed);
    }

    #[test]
    fn test_with_spaces() {
        check(
            Strategy::Title,
            Paper {
                title: Some("my long title with spaces".to_owned()),
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
            Paper {
                title: Some("MLT: my |long<> title\" with/ spaces and * more?".to_owned()),
                ..Default::default()
            },
            expect!["MLT my long title with spaces and  more"],
        );
    }
}
