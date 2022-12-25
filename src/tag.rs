use std::{fmt::Display, str::FromStr};

use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Tag {
    key: String,
}

impl Tag {
    pub fn new(key: &str) -> Self {
        let key = key.trim();
        assert!(
            !key.contains(char::is_whitespace),
            "Tag key contains whitespace"
        );
        Self {
            key: key.to_owned(),
        }
    }

    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }
}

impl FromStr for Tag {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self::new(s))
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.key)
    }
}
