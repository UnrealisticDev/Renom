use std::{fmt::Display, path::PathBuf};

use colored::Colorize;

#[derive(Debug, PartialEq)]
pub struct SetIniEntry {
    pub path: PathBuf,
    pub section: String,
    pub key: String,
    pub value: String,
}

impl SetIniEntry {
    pub fn new(
        path: impl Into<PathBuf>,
        section: impl Into<String>,
        key: impl Into<String>,
        value: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            section: section.into(),
            key: key.into(),
            value: value.into(),
        }
    }
}

impl Display for SetIniEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "set [{}] {} = {} in config file {}",
            &self.section.dimmed(),
            &self.key.dimmed(),
            &self.value.dimmed(),
            &self
                .path
                .to_str()
                .unwrap_or("invalid Unicode path")
                .dimmed()
        )
    }
}
