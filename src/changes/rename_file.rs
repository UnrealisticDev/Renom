use std::{fmt::Display, path::PathBuf};

use colored::Colorize;

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
            "rename file {} to {}",
            &self
                .from
                .to_str()
                .unwrap_or("invalid Unicode path")
                .dimmed(),
            &self.to.to_str().unwrap_or("invalid Unicode path").dimmed()
        )
    }
}
