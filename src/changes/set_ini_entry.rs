use std::{fmt::Display, path::PathBuf};

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
            "Set section [{}], key [{}] to value [{}] in INI file [{}]",
            &self.section,
            &self.key,
            &self.value,
            &self.path.to_str().unwrap()
        )
    }
}
