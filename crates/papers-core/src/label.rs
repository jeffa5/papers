use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

use crate::primitive::Primitive;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Label {
    key: String,
    value: Primitive,
}

impl PartialOrd for Label {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for Label {}

impl Ord for Label {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.key.cmp(&other.key)
    }
}

impl Label {
    pub fn new(key: &str, value: Primitive) -> Self {
        let key = key.trim();
        assert!(
            !key.contains(char::is_whitespace),
            "Label key contains whitespace"
        );
        Self {
            key: key.to_owned(),
            value,
        }
    }

    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    #[must_use]
    pub fn value(&self) -> &Primitive {
        &self.value
    }
}

impl FromStr for Label {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kv = s.split('=').collect::<Vec<_>>();
        match kv[..] {
            [k, v] => Ok(Self::new(
                k,
                v.parse()
                    .unwrap_or_else(|_| Primitive::String(v.to_owned())),
            )),
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
