use std::{path::PathBuf, str::FromStr};

use reqwest::Url;

/// Url or path.
#[derive(Debug, Clone)]
pub enum UrlOrPath {
    /// The url type, with http or https scheme.
    Url(Url),
    /// Local file path.
    Path(PathBuf),
}

impl FromStr for UrlOrPath {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Url::parse(s) {
            Ok(url) => match url.scheme() {
                "http" | "https" => Ok(Self::Url(url)),
                "file" => match url.to_file_path() {
                    Ok(path) => Ok(Self::Path(path)),
                    Err(_) => Err("failed to parse string as valid url or path".to_owned()),
                },
                _ => Err(format!(
                    "failed to parse string as valid url or path {}",
                    url
                )),
            },
            Err(_) => Ok(Self::Path(PathBuf::from(s))),
        }
    }
}
