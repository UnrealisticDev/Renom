use std::path::PathBuf;

use crate::unreal::Module;

/// Context needed to rename an Unreal Engine module.
pub struct Context {
    /// The root of the project that the module is part of.
    pub project_root: PathBuf,
    /// The name of the project.
    pub project_name: String,
    /// The specific module to rename.
    pub target_module: Module,
    /// The target name for the module.
    pub target_name: String,
}
