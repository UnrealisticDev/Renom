use std::{fmt::Display, path::PathBuf};

#[derive(Clone)]
pub enum PluginType {
    Project,
    Plugin,
}

/// Information about an Unreal Engine module.
#[derive(Clone)]
pub struct Module {
    /// The name of the module.
    pub name: String,
    // The path to the root of the module.
    pub root: PathBuf,
    /// The type of the module.
    pub r#type: PluginType,
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}
