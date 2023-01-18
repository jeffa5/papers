use std::{path::PathBuf, str::FromStr};

/// A filename or stdin.
#[derive(Debug, Clone)]
pub enum FileOrStdin {
    /// A filename.
    File(PathBuf),
    /// stdin.
    Stdin,
}

impl FromStr for FileOrStdin {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "-" {
            Ok(Self::Stdin)
        } else {
            let path = PathBuf::from(s);
            if path.is_file() {
                Ok(Self::File(path))
            } else {
                Err(format!("Path was not a file: {:?}", path))
            }
        }
    }
}
