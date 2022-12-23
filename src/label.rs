use std::{str::FromStr, fmt::Display};

#[derive(Debug, Clone, PartialEq)]
pub struct Label {
    pub key: String,
    pub value: String,
}

impl FromStr for Label {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let kv = s.split('=').collect::<Vec<_>>();
        match kv[..] {
            [k, v] => Ok(Label {
                key: k.to_owned(),
                value: v.to_owned(),
            }),
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