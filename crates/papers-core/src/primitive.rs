use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Primitive {
    Null,
    Bool(bool),
    Number(serde_yaml::value::Number),
    String(String),
}

impl Display for Primitive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Null => "".to_owned(),
                Self::Bool(b) => b.to_string(),
                Self::Number(n) => n.to_string(),
                Self::String(s) => s.to_owned(),
            }
        )
    }
}

impl FromStr for Primitive {
    type Err = serde_yaml::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_yaml::from_str(s)
    }
}
