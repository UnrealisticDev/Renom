use std::path::PathBuf;

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
