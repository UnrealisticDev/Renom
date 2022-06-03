use std::{fmt::Display, path::PathBuf};

#[derive(Debug, PartialEq)]
pub struct RenameFile {
    pub from: PathBuf,
    pub to: PathBuf,
}

impl RenameFile {
    pub fn new(from: impl Into<PathBuf>, to: impl Into<PathBuf>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
        }
    }
}

impl Display for RenameFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Rename [{}] to [{}]",
            &self.from.to_str().unwrap_or("invalid Unicode path"),
            &self.to.to_str().unwrap_or("invalid Unicode path")
        )
    }
}
