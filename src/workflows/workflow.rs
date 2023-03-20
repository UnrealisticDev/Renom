use std::fmt::Display;

pub enum Workflow {
    RenameProject,
    RenamePlugin,
    RenameTarget,
    RenameModule,
}

impl Display for Workflow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &*self {
            Workflow::RenameProject => write!(f, "Rename a project"),
            Workflow::RenamePlugin => write!(f, "Rename a plugin"),
            Workflow::RenameTarget => write!(f, "Rename a target"),
            Workflow::RenameModule => write!(f, "Rename a module"),
        }
    }
}
