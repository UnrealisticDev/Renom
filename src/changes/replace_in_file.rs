use std::path::PathBuf;

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
