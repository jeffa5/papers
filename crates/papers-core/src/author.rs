use std::{fmt::Display, str::FromStr};

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Author {
    author: String,
}

impl PartialOrd for Author {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Author {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.author.cmp(&other.author)
    }
}

impl Author {
    pub fn new(s: &str) -> Self {
        Self {
            author: s.trim().to_owned(),
        }
    }
}

impl FromStr for Author {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.author)
    }
}
