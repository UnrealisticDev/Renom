use std::path::PathBuf;

pub enum ProjectType {
    Blueprint,
    Code,
}

/// Context needed to rename an Unreal Engine project.
pub struct Context {
    /// The root of the project.
    pub project_root: PathBuf,
    /// The name of the project.
    pub project_name: String,
    /// The type of the project.
    pub project_type: ProjectType,
    /// The target name for the project.
    pub target_name: String,
}
