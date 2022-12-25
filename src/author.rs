use std::{fmt::Display, str::FromStr};

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Author {
    author: String,
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
