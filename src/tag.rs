use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub struct Tag {
    key: String,
}

impl Tag {
    pub fn new(key: &str) -> Self {
        let key = key.trim();
        if key.contains(char::is_whitespace) {
            panic!("Tag key contains whitespace");
        }
        Tag {
            key: key.to_owned(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }
}

impl FromStr for Tag {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Tag { key: s.to_owned() })
    }
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.key)
    }
}
