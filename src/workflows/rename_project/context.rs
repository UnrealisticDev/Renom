use std::path::PathBuf;

/// Context needed to rename an Unreal Engine project.
pub struct Context {
    /// The root of the project.
    pub project_root: PathBuf,
    /// The name of the project.
    pub project_name: String,
    /// The target name for the project.
    pub target_name: String,
}
