use std::path::PathBuf;

#[derive(Debug, PartialEq)]
pub enum Change {
    RenameFile(RenameFile),
    AddEntryToIni(AddEntryToIni),
}

#[derive(Debug, PartialEq)]
pub struct RenameFile {
    from: PathBuf,
    to: PathBuf,
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
pub struct AddEntryToIni {
    path: PathBuf,
    section: String,
    key: String,
    value: String,
}

impl AddEntryToIni {
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
