use std::{fmt::Display, str::FromStr};

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    key: String,
    value: String,
}

impl Label {
    pub fn new(key: &str, value: &str) -> Self {
        Label {
            key: key.trim().to_owned(),
            value: value.trim().to_owned(),
        }
    }

    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl FromStr for Label {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kv = s.split('=').collect::<Vec<_>>();
        match kv[..] {
            [k, v] => Ok(Label::new(k, v)),
            [_] => Err("Missing value, should be of the form `key=value`"),
            _ => Err("Too many `=`, should be of the form `key=value`"),
        }
    }
}

impl Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}={}", self.key, self.value)
    }
}
