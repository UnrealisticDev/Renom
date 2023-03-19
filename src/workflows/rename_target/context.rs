use std::path::PathBuf;

use crate::unreal::Target;

/// Context needed to rename an Unreal Engine target.
pub struct Context {
    /// The root of the project.
    pub project_root: PathBuf,
    /// Build targets for the project.
    pub project_targets: Vec<Target>,
    /// The specific target to rename.
    pub target_target: Target,
    /// The target name for the target.
    pub target_name: String,
}
