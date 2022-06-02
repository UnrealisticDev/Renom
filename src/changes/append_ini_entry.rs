use std::path::PathBuf;

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
