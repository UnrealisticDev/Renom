use std::{fmt::Display, path::PathBuf};

#[derive(Clone)]
pub enum ModuleType {
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
    pub r#type: ModuleType,
    /// The host plugin (for plugin modules).
    pub plugin: Option<Plugin>,
}

impl Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}

/// Information about an Unreal Engine plugin.
#[derive(Clone)]
pub struct Plugin {
    /// The name of the plugin.
    pub name: String,
    /// The path to the root of the plugin.
    pub root: PathBuf,
}

impl Display for Plugin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}

/// Information about an Unreal Engine target.
#[derive(Clone)]
pub struct Target {
    /// The name of the target.
    pub name: String,
    // The path to the target file.
    pub path: PathBuf,
}

impl Display for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.name)
    }
}
