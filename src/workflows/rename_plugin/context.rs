use std::path::PathBuf;

use crate::unreal::Plugin;

/// Context needed to rename an Unreal Engine plugin.
pub struct Context {
    /// The root of the project.
    pub project_root: PathBuf,
    /// The name of the project.
    pub project_name: String,
    /// Plugins for the project.
    pub project_plugins: Vec<Plugin>,
    /// The specific plugin to rename.
    pub target_plugin: Plugin,
    /// The target name for the plugin.
    pub target_name: String,
}
