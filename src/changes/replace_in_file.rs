use std::{fmt::Display, path::PathBuf};

use colored::Colorize;

#[derive(Debug, PartialEq)]
pub struct ReplaceInFile {
    pub path: PathBuf,
    pub from: String,
    pub to: String,
}

impl ReplaceInFile {
    pub fn new(path: impl Into<PathBuf>, from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            from: from.into(),
            to: to.into(),
        }
    }
}

impl Display for ReplaceInFile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "replace {} with {} in file {}",
            &self.from.dimmed(),
            &self.to.dimmed(),
            &self
                .path
                .to_str()
                .unwrap_or("invalid Unicode path")
                .dimmed()
        )
    }
}
