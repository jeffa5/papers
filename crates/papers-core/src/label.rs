use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    key: String,
    value: String,
}

impl PartialOrd for Label {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Label {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl Label {
    pub fn new(key: &str, value: &str) -> Self {
        let key = key.trim();
        assert!(
            !key.contains(char::is_whitespace),
            "Label key contains whitespace"
        );
        let value = value.trim();
        assert!(
            !value.contains(char::is_whitespace),
            "Label value contains whitespace"
        );
        Self {
            key: key.to_owned(),
            value: value.to_owned(),
        }
    }

    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl FromStr for Label {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kv = s.split('=').collect::<Vec<_>>();
        match kv[..] {
            [k, v] => Ok(Self::new(k, v)),
            [_] => Err("Missing value, should be of the form `key=value`"),
            _ => Err("Too many `=`, should be of the form `key=value`"),
        }
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}={}", self.key, self.value)
    }
}
