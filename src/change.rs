use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum Change {
    RenameFile(RenameFile),
    ReplaceInFile(ReplaceInFile),
    SetIniEntry(SetIniEntry),
    AppendIniEntry(AppendIniEntry),
}

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

#[derive(Debug, PartialEq)]
pub struct AppendIniEntry {
    pub path: PathBuf,
    pub section: String,
    pub key: String,
    pub value: String,
}

impl AppendIniEntry {
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
