use std::{fmt::Display, path::PathBuf};

/// Information about an Unreal Engine module.
#[derive(Clone)]
pub struct Module {
    /// The name of the module.
    pub name: String,
    // The path to the root of the module.
    pub root: PathBuf,
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}
